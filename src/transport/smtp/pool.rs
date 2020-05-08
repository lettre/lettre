use crate::transport::smtp::{client::SmtpConnection, error::Error, SmtpClient};
use r2d2::ManageConnection;

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
