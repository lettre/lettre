use async_trait::async_trait;

use super::client::AsyncSmtpConnection;
#[cfg(feature = "tokio02")]
use super::Tls;
use super::{ClientId, Credentials, Error, Mechanism, Response, SmtpInfo};
use crate::{Envelope, Tokio02Transport};

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AsyncSmtpTransport<C> {
    // TODO: pool
    inner: AsyncSmtpClient<C>,
}

#[async_trait]
impl Tokio02Transport for AsyncSmtpTransport<Tokio02Connector> {
    type Ok = Response;
    type Error = Error;

    /// Sends an email
    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut conn = self.inner.connection().await?;

        let result = conn.send(envelope, email).await?;

        conn.quit().await?;

        Ok(result)
    }
}

impl<C> AsyncSmtpTransport<C>
where
    C: AsyncSmtpConnector,
{
    /// Simple and secure transport, should be used when possible.
    /// Creates an encrypted transport over submissions port, using the provided domain
    /// to validate TLS certificates.
    #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
    pub fn relay(relay: &str) -> Result<AsyncSmtpTransportBuilder, Error> {
        use super::{TlsParameters, SUBMISSIONS_PORT};

        let tls_parameters = TlsParameters::new_tokio02(relay.into())?;

        Ok(Self::builder_dangerous(relay)
            .port(SUBMISSIONS_PORT)
            .tls(Tls::Wrapper(tls_parameters)))
    }

    /// Simple and secure transport, should be used when the server doesn't support wrapped TLS connections.
    /// Creates an encrypted transport over submissions port, by first connecting using an unencrypted
    /// connection and then upgrading it with STARTTLS, using the provided domain to validate TLS certificates.
    /// If the connection can't be upgraded it will fail connecting altogether.
    #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
    pub fn starttls_relay(relay: &str) -> Result<AsyncSmtpTransportBuilder, Error> {
        use super::{TlsParameters, SUBMISSION_PORT};

        let tls_parameters = TlsParameters::new(relay.into())?;

        Ok(Self::builder_dangerous(relay)
            .port(SUBMISSION_PORT)
            .tls(Tls::Required(tls_parameters)))
    }

    /// Creates a new local SMTP client to port 25
    ///
    /// Shortcut for local unencrypted relay (typical local email daemon that will handle relaying)
    pub fn unencrypted_localhost() -> AsyncSmtpTransport<C> {
        Self::builder_dangerous("localhost").build()
    }

    /// Creates a new SMTP client
    ///
    /// Defaults are:
    ///
    /// * No authentication
    /// * No TLS
    /// * Port 25
    ///
    /// Consider using [`AsyncSmtpTransport::relay`] instead, if possible.
    pub fn builder_dangerous<T: Into<String>>(server: T) -> AsyncSmtpTransportBuilder {
        let mut new = SmtpInfo::default();
        new.server = server.into();
        AsyncSmtpTransportBuilder { info: new }
    }
}

/// Contains client configuration
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AsyncSmtpTransportBuilder {
    info: SmtpInfo,
}

/// Builder for the SMTP `AsyncSmtpTransport`
impl AsyncSmtpTransportBuilder {
    /// Set the name used during EHLO
    pub fn hello_name(mut self, name: ClientId) -> Self {
        self.info.hello_name = name;
        self
    }

    /// Set the authentication mechanism to use
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.info.credentials = Some(credentials);
        self
    }

    /// Set the authentication mechanism to use
    pub fn authentication(mut self, mechanisms: Vec<Mechanism>) -> Self {
        self.info.authentication = mechanisms;
        self
    }

    /// Set the port to use
    pub fn port(mut self, port: u16) -> Self {
        self.info.port = port;
        self
    }

    /// Set the TLS settings to use
    #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
    pub fn tls(mut self, tls: Tls) -> Self {
        self.info.tls = tls;
        self
    }

    /// Build the transport (with default pool if enabled)
    pub fn build<C>(self) -> AsyncSmtpTransport<C>
    where
        C: AsyncSmtpConnector,
    {
        let connector = Default::default();
        let client = AsyncSmtpClient {
            connector,
            info: self.info,
        };
        AsyncSmtpTransport { inner: client }
    }
}

/// Build client
#[derive(Clone)]
pub struct AsyncSmtpClient<C> {
    connector: C,
    info: SmtpInfo,
}

impl<C> AsyncSmtpClient<C>
where
    C: AsyncSmtpConnector,
{
    /// Creates a new connection directly usable to send emails
    ///
    /// Handles encryption and authentication
    pub async fn connection(&self) -> Result<AsyncSmtpConnection, Error> {
        let mut conn = C::connect(
            &self.info.server,
            self.info.port,
            &self.info.hello_name,
            &self.info.tls,
        )
        .await?;

        if let Some(credentials) = &self.info.credentials {
            conn.auth(&self.info.authentication, &credentials).await?;
        }
        Ok(conn)
    }
}

#[async_trait]
pub trait AsyncSmtpConnector: Default + private::Sealed {
    async fn connect(
        hostname: &str,
        port: u16,
        hello_name: &ClientId,
        tls: &Tls,
    ) -> Result<AsyncSmtpConnection, Error>;
}

#[derive(Debug, Copy, Clone, Default)]
#[cfg(feature = "tokio02")]
pub struct Tokio02Connector;

#[async_trait]
#[cfg(feature = "tokio02")]
impl AsyncSmtpConnector for Tokio02Connector {
    async fn connect(
        hostname: &str,
        port: u16,
        hello_name: &ClientId,
        tls: &Tls,
    ) -> Result<AsyncSmtpConnection, Error> {
        #[allow(clippy::match_single_binding)]
        let tls_parameters = match tls {
            #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
            Tls::Wrapper(ref tls_parameters) => Some(tls_parameters.clone()),
            _ => None,
        };
        let mut conn =
            AsyncSmtpConnection::connect_tokio02(hostname, port, hello_name, tls_parameters)
                .await?;

        #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
        match tls {
            Tls::Opportunistic(ref tls_parameters) => {
                if conn.can_starttls() {
                    conn.starttls(tls_parameters.clone(), hello_name).await?;
                }
            }
            Tls::Required(ref tls_parameters) => {
                conn.starttls(tls_parameters.clone(), hello_name).await?;
            }
            _ => (),
        }

        Ok(conn)
    }
}

mod private {
    use super::*;

    pub trait Sealed {}

    #[cfg(feature = "tokio02")]
    impl Sealed for Tokio02Connector {}
}
