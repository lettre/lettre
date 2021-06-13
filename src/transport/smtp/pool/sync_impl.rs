use r2d2::{CustomizeConnection, ManageConnection, Pool};

use super::PoolConfig;
use crate::transport::smtp::{client::SmtpConnection, error, error::Error, SmtpClient};

impl PoolConfig {
    pub(crate) fn build<C: ManageConnection<Connection = SmtpConnection, Error = Error>>(
        &self,
        client: C,
    ) -> Pool<C> {
        Pool::builder()
            .min_idle(Some(self.min_idle))
            .max_size(self.max_size)
            .connection_timeout(self.connection_timeout)
            .idle_timeout(Some(self.idle_timeout))
            .connection_customizer(Box::new(SmtpConnectionQuitter))
            .build_unchecked(client)
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
        Err(error::network("is not connected anymore"))
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.has_broken()
    }
}

#[derive(Copy, Clone, Debug)]
struct SmtpConnectionQuitter;

impl CustomizeConnection<SmtpConnection, Error> for SmtpConnectionQuitter {
    fn on_release(&self, conn: SmtpConnection) {
        let mut conn = conn;
        if !conn.has_broken() {
            let _quit = conn.quit();
        }
    }
}
