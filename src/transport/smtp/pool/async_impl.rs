use std::{
    fmt::{self, Debug},
    mem,
    ops::{Deref, DerefMut},
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use futures_util::{
    lock::Mutex,
    stream::{self, StreamExt},
};

use super::{
    super::{client::AsyncSmtpConnection, Error},
    PoolConfig,
};
use crate::{executor::SpawnHandle, transport::smtp::async_transport::AsyncSmtpClient, Executor};

pub(crate) struct Pool<E: Executor> {
    config: PoolConfig,
    connections: Mutex<Vec<ParkedConnection>>,
    client: AsyncSmtpClient<E>,
    handle: OnceLock<E::Handle>,
}

struct ParkedConnection {
    conn: AsyncSmtpConnection,
    since: Instant,
}

pub(crate) struct PooledConnection<E: Executor> {
    conn: Option<AsyncSmtpConnection>,
    pool: Arc<Pool<E>>,
}

impl<E: Executor> Pool<E> {
    pub(crate) fn new(config: PoolConfig, client: AsyncSmtpClient<E>) -> Arc<Self> {
        let pool = Arc::new(Self {
            config,
            connections: Mutex::new(Vec::new()),
            client,
            handle: OnceLock::new(),
        });

        {
            let pool_ = Arc::clone(&pool);

            let min_idle = pool_.config.min_idle;
            let idle_timeout = pool_.config.idle_timeout;
            let pool = Arc::downgrade(&pool_);

            let handle = E::spawn(async move {
                loop {
                    #[cfg(feature = "tracing")]
                    tracing::trace!("running cleanup tasks");

                    match pool.upgrade() {
                        Some(pool) => {
                            #[allow(clippy::needless_collect)]
                            let (count, dropped) = {
                                let mut connections = pool.connections.lock().await;

                                let to_drop = connections
                                    .iter()
                                    .enumerate()
                                    .rev()
                                    .filter(|(_, conn)| conn.idle_duration() > idle_timeout)
                                    .map(|(i, _)| i)
                                    .collect::<Vec<_>>();
                                let dropped = to_drop
                                    .into_iter()
                                    .map(|i| connections.remove(i))
                                    .collect::<Vec<_>>();

                                (connections.len(), dropped)
                            };

                            #[cfg(feature = "tracing")]
                            let mut created = 0;
                            for _ in count..(min_idle as usize) {
                                let conn = match pool.client.connection().await {
                                    Ok(conn) => conn,
                                    Err(err) => {
                                        #[cfg(feature = "tracing")]
                                        tracing::warn!("couldn't create idle connection {}", err);
                                        #[cfg(not(feature = "tracing"))]
                                        let _ = err;

                                        break;
                                    }
                                };

                                let mut connections = pool.connections.lock().await;
                                connections.push(ParkedConnection::park(conn));

                                #[cfg(feature = "tracing")]
                                {
                                    created += 1;
                                }
                            }

                            #[cfg(feature = "tracing")]
                            if created > 0 {
                                tracing::debug!("created {} idle connections", created);
                            }

                            if !dropped.is_empty() {
                                #[cfg(feature = "tracing")]
                                tracing::debug!("dropped {} idle connections", dropped.len());

                                abort_concurrent(dropped.into_iter().map(ParkedConnection::unpark))
                                    .await;
                            }
                        }
                        None => {
                            #[cfg(feature = "tracing")]
                            tracing::warn!(
                                "breaking out of task - no more references to Pool are available"
                            );
                            break;
                        }
                    }

                    E::sleep(idle_timeout).await;
                }
            });
            pool_
                .handle
                .set(handle)
                .expect("handle hasn't been set yet");
        }

        pool
    }

    pub(crate) async fn connection(self: &Arc<Self>) -> Result<PooledConnection<E>, Error> {
        loop {
            let conn = {
                let mut connections = self.connections.lock().await;
                connections.pop()
            };

            match conn {
                Some(conn) => {
                    let mut conn = conn.unpark();

                    // TODO: handle the client try another connection if this one isn't good
                    if !conn.test_connected().await {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("dropping a broken connection");

                        conn.abort().await;
                        continue;
                    }

                    #[cfg(feature = "tracing")]
                    tracing::debug!("reusing a pooled connection");

                    return Ok(PooledConnection::wrap(conn, Arc::clone(self)));
                }
                None => {
                    #[cfg(feature = "tracing")]
                    tracing::debug!("creating a new connection");

                    let conn = self.client.connection().await?;
                    return Ok(PooledConnection::wrap(conn, Arc::clone(self)));
                }
            }
        }
    }

    async fn recycle(&self, mut conn: AsyncSmtpConnection) {
        if conn.has_broken() {
            #[cfg(feature = "tracing")]
            tracing::debug!("dropping a broken connection instead of recycling it");

            conn.abort().await;
            drop(conn);
        } else {
            #[cfg(feature = "tracing")]
            tracing::debug!("recycling connection");

            let mut connections = self.connections.lock().await;
            if connections.len() >= self.config.max_size as usize {
                drop(connections);
                conn.abort().await;
            } else {
                let conn = ParkedConnection::park(conn);
                connections.push(conn);
            }
        }
    }
}

impl<E: Executor> Debug for Pool<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pool")
            .field("config", &self.config)
            .field(
                "connections",
                &match self.connections.try_lock() {
                    Some(connections) => format!("{} connections", connections.len()),

                    None => "LOCKED".to_owned(),
                },
            )
            .field("client", &self.client)
            .field(
                "handle",
                &match self.handle.get() {
                    Some(_) => "Some(JoinHandle)",
                    None => "None",
                },
            )
            .finish()
    }
}

impl<E: Executor> Drop for Pool<E> {
    fn drop(&mut self) {
        #[cfg(feature = "tracing")]
        tracing::debug!("dropping Pool");

        let connections = mem::take(self.connections.get_mut());
        let handle = self.handle.take();
        E::spawn(async move {
            if let Some(handle) = handle {
                handle.shutdown().await;
            }

            abort_concurrent(connections.into_iter().map(ParkedConnection::unpark)).await;
        });
    }
}

impl ParkedConnection {
    fn park(conn: AsyncSmtpConnection) -> Self {
        Self {
            conn,
            since: Instant::now(),
        }
    }

    fn idle_duration(&self) -> Duration {
        self.since.elapsed()
    }

    fn unpark(self) -> AsyncSmtpConnection {
        self.conn
    }
}

impl<E: Executor> PooledConnection<E> {
    fn wrap(conn: AsyncSmtpConnection, pool: Arc<Pool<E>>) -> Self {
        Self {
            conn: Some(conn),
            pool,
        }
    }
}

impl<E: Executor> Deref for PooledConnection<E> {
    type Target = AsyncSmtpConnection;

    fn deref(&self) -> &Self::Target {
        self.conn.as_ref().expect("conn hasn't been dropped yet")
    }
}

impl<E: Executor> DerefMut for PooledConnection<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn.as_mut().expect("conn hasn't been dropped yet")
    }
}

impl<E: Executor> Drop for PooledConnection<E> {
    fn drop(&mut self) {
        let conn = self
            .conn
            .take()
            .expect("AsyncSmtpConnection hasn't been taken yet");
        let pool = Arc::clone(&self.pool);

        E::spawn(async move {
            pool.recycle(conn).await;
        });
    }
}

async fn abort_concurrent<I>(iter: I)
where
    I: Iterator<Item = AsyncSmtpConnection>,
{
    stream::iter(iter)
        .for_each_concurrent(8, |mut conn| async move {
            conn.abort().await;
        })
        .await;
}
