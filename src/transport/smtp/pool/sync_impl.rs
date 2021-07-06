use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, TryLockError};
use std::time::{Duration, Instant};
use std::{mem, thread};

use crate::transport::smtp::transport::SmtpClient;

use super::super::client::SmtpConnection;
use super::super::Error;
use super::PoolConfig;

pub struct Pool {
    config: PoolConfig,
    connections: Mutex<Vec<ParkedConnection>>,
    client: SmtpClient,
}

struct ParkedConnection {
    conn: SmtpConnection,
    since: Instant,
}

pub struct PooledConnection {
    conn: Option<SmtpConnection>,
    pool: Arc<Pool>,
}

impl Pool {
    pub fn new(config: PoolConfig, client: SmtpClient) -> Arc<Self> {
        let pool = Arc::new(Self {
            config,
            connections: Mutex::new(Vec::new()),
            client,
        });

        {
            let pool_ = Arc::clone(&pool);

            let min_idle = pool_.config.min_idle;
            let idle_timeout = pool_.config.idle_timeout;
            let pool = Arc::downgrade(&pool_);

            thread::Builder::new()
                .name("lettre-connection-pool".into())
                .spawn(move || {
                    while let Some(pool) = pool.upgrade() {
                        #[cfg(feature = "tracing")]
                        tracing::trace!("running cleanup tasks");

                        #[allow(clippy::needless_collect)]
                        let (count, dropped) = {
                            let mut connections = pool.connections.lock().unwrap();

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
                        for _ in count..=(min_idle as usize) {
                            let conn = match pool.client.connection() {
                                Ok(conn) => conn,
                                Err(err) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::warn!("couldn't create idle connection {}", err);
                                    #[cfg(not(feature = "tracing"))]
                                    let _ = err;

                                    break;
                                }
                            };

                            let mut connections = pool.connections.lock().unwrap();
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

                            for conn in dropped {
                                let mut conn = conn.unpark();
                                conn.abort();
                            }
                        }

                        thread::sleep(idle_timeout);
                    }
                })
                .expect("couldn't spawn the Pool thread");
        }

        pool
    }

    pub fn connection(self: &Arc<Self>) -> Result<PooledConnection, Error> {
        loop {
            let conn = {
                let mut connections = self.connections.lock().unwrap();
                connections.pop()
            };

            match conn {
                Some(conn) => {
                    let mut conn = conn.unpark();

                    // TODO: handle the client try another connection if this one isn't good
                    if !conn.test_connected() {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("dropping a broken connection");

                        conn.abort();
                        continue;
                    }

                    #[cfg(feature = "tracing")]
                    tracing::debug!("reusing a pooled connection");

                    return Ok(PooledConnection::wrap(conn, self.clone()));
                }
                None => {
                    #[cfg(feature = "tracing")]
                    tracing::debug!("creating a new connection");

                    let conn = self.client.connection()?;
                    return Ok(PooledConnection::wrap(conn, self.clone()));
                }
            }
        }
    }

    fn recycle(&self, mut conn: SmtpConnection) {
        if conn.has_broken() {
            #[cfg(feature = "tracing")]
            tracing::debug!("dropping a broken connection instead of recycling it");

            conn.abort();
            drop(conn);
        } else {
            #[cfg(feature = "tracing")]
            tracing::debug!("recycling connection");

            let mut connections = self.connections.lock().unwrap();
            if connections.len() >= self.config.max_size as usize {
                drop(connections);
                conn.abort();
            } else {
                let conn = ParkedConnection::park(conn);
                connections.push(conn);
            }
        }
    }
}

impl Debug for Pool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pool")
            .field("config", &self.config)
            .field(
                "connections",
                &match self.connections.try_lock() {
                    Ok(connections) => format!("{} connections", connections.len()),

                    Err(TryLockError::WouldBlock) => "LOCKED".to_string(),
                    Err(TryLockError::Poisoned(_)) => "POISONED".to_string(),
                },
            )
            .field("client", &self.client)
            .finish()
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        #[cfg(feature = "tracing")]
        tracing::debug!("dropping Pool");

        let connections = mem::take(&mut *self.connections.get_mut().unwrap());
        for conn in connections {
            let mut conn = conn.unpark();
            conn.abort();
        }
    }
}

impl ParkedConnection {
    fn park(conn: SmtpConnection) -> Self {
        Self {
            conn,
            since: Instant::now(),
        }
    }

    fn idle_duration(&self) -> Duration {
        self.since.elapsed()
    }

    fn unpark(self) -> SmtpConnection {
        self.conn
    }
}

impl PooledConnection {
    fn wrap(conn: SmtpConnection, pool: Arc<Pool>) -> Self {
        Self {
            conn: Some(conn),
            pool,
        }
    }
}

impl Deref for PooledConnection {
    type Target = SmtpConnection;

    fn deref(&self) -> &Self::Target {
        self.conn.as_ref().expect("conn hasn't been dropped yet")
    }
}

impl DerefMut for PooledConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn.as_mut().expect("conn hasn't been dropped yet")
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        let conn = self
            .conn
            .take()
            .expect("SmtpConnection hasn't been taken yet");
        self.pool.recycle(conn);
    }
}
