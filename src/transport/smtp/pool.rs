use std::time::Duration;

use crate::transport::smtp::{client::SmtpConnection, error::Error, SmtpClient};

use r2d2::{ManageConnection, Pool};

/// Configuration for a connection pool
#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
#[cfg_attr(docsrs, doc(cfg(feature = "r2d2")))]
pub struct PoolConfig {
    min_idle: u32,
    max_size: u32,
    connection_timeout: Duration,
    idle_timeout: Duration,
}

impl PoolConfig {
    /// Minimum number of idle connections
    ///
    /// Defaults to `0`
    pub fn min_idle(mut self, min_idle: u32) -> Self {
        self.min_idle = min_idle;
        self
    }

    /// Maximum number of pooled connections
    ///
    /// Defaults to `10`
    pub fn max_size(mut self, max_size: u32) -> Self {
        self.min_idle = max_size;
        self
    }

    /// Connection timeout
    ///
    /// Defaults to `30 seconds`
    pub fn connection_timeout(mut self, connection_timeout: Duration) -> Self {
        self.connection_timeout = connection_timeout;
        self
    }

    /// Connection idle timeout
    ///
    /// Defaults to `60 seconds`
    pub fn idle_timeout(mut self, idle_timeout: Duration) -> Self {
        self.idle_timeout = idle_timeout;
        self
    }

    pub(crate) fn build<C: ManageConnection>(&self, client: C) -> Pool<C> {
        Pool::builder()
            .min_idle(Some(self.min_idle))
            .max_size(self.max_size)
            .connection_timeout(self.connection_timeout)
            .idle_timeout(Some(self.idle_timeout))
            .build_unchecked(client)
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_idle: 0,
            max_size: 10,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(60),
        }
    }
}

impl ManageConnection for SmtpClient {
    type Connection = SmtpConnection;
    type Error = Error;

    fn connect(&self) -> Result<Self::Connection, Error> {
        self.connection()
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Error> {
        if conn.test_connected() {
            return Ok(());
        }
        Err(Error::Client("is not connected anymore"))
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.has_broken()
    }
}
