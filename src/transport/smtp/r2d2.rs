use crate::transport::smtp::{
    error::Error, ConnectionReuseParameters, SmtpTransport, SmtpTransport,
};
use r2d2::ManageConnection;

pub struct SmtpConnectionManager {
    transport_builder: SmtpTransport,
}

impl SmtpConnectionManager {
    pub fn new(transport_builder: SmtpTransport) -> Result<SmtpConnectionManager, Error> {
        Ok(SmtpConnectionManager {
            transport_builder: transport_builder
                .connection_reuse(ConnectionReuseParameters::ReuseUnlimited),
        })
    }
}

impl ManageConnection for SmtpConnectionManager {
    type Connection = SmtpTransport;
    type Error = Error;

    fn connect(&self) -> Result<Self::Connection, Error> {
        let mut transport = SmtpTransport::new(self.transport_builder.clone());
        transport.connect()?;
        Ok(transport)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Error> {
        if conn.client.test_connected() {
            return Ok(());
        }
        Err(Error::Client("is not connected anymore"))
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.state.panic
    }
}
