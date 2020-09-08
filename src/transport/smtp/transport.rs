use std::time::Duration;

#[cfg(feature = "r2d2")]
use r2d2::{Builder, Pool};

use super::{ClientId, Credentials, Error, Mechanism, Response, SmtpConnection, SmtpInfo};
#[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
use super::{Tls, TlsParameters, SUBMISSIONS_PORT, SUBMISSION_PORT};
use crate::{Envelope, Transport};

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct SmtpTransport {
    #[cfg(feature = "r2d2")]
    inner: Pool<SmtpClient>,
    #[cfg(not(feature = "r2d2"))]
    inner: SmtpClient,
}

impl Transport for SmtpTransport {
    type Ok = Response;
    type Error = Error;

    /// Sends an email
    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        #[cfg(feature = "r2d2")]
        let mut conn = self.inner.get()?;
        #[cfg(not(feature = "r2d2"))]
        let mut conn = self.inner.connection()?;

        let result = conn.send(envelope, email)?;

        #[cfg(not(feature = "r2d2"))]
        conn.quit()?;

        Ok(result)
    }
}

impl SmtpTransport {
    /// Simple and secure transport, using TLS connections to comunicate with the SMTP server
    ///
    /// The right option for most SMTP servers.
    ///
    /// Creates an encrypted transport over submissions port, using the provided domain
    /// to validate TLS certificates.
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    pub fn relay(relay: &str) -> Result<SmtpTransportBuilder, Error> {
        let tls_parameters = TlsParameters::new(relay.into())?;

        Ok(Self::builder_dangerous(relay)
            .port(SUBMISSIONS_PORT)
            .tls(Tls::Wrapper(tls_parameters)))
    }

    /// Simple an secure transport, using STARTTLS to obtain encrypted connections
    ///
    /// Alternative to [`SmtpTransport::relay`](#method.relay), for SMTP servers
    /// that don't take SMTPS connections.
    ///
    /// Creates an encrypted transport over submissions port, by first connecting using
    /// an unencrypted connection and then upgrading it with STARTTLS. The provided
    /// domain is used to validate TLS certificates.
    ///
    /// An error is returned if the connection can't be upgraded. No credentials
    /// or emails will be sent to the server, protecting from downgrade attacks.
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    pub fn starttls_relay(relay: &str) -> Result<SmtpTransportBuilder, Error> {
        let tls_parameters = TlsParameters::new(relay.into())?;

        Ok(Self::builder_dangerous(relay)
            .port(SUBMISSION_PORT)
            .tls(Tls::Required(tls_parameters)))
    }

    /// Creates a new local SMTP client to port 25
    ///
    /// Shortcut for local unencrypted relay (typical local email daemon that will handle relaying)
    pub fn unencrypted_localhost() -> SmtpTransport {
        Self::builder_dangerous("localhost").build()
    }

    /// Creates a new SMTP client
    ///
    /// Defaults are:
    ///
    /// * No authentication
    /// * No TLS
    /// * A 60 seconds timeout for smtp commands
    /// * Port 25
    ///
    /// Consider using [`SmtpTransport::relay`](#method.relay) or
    /// [`SmtpTransport::starttls_relay`](#method.starttls_relay) instead,
    /// if possible.
    pub fn builder_dangerous<T: Into<String>>(server: T) -> SmtpTransportBuilder {
        let mut new = SmtpInfo::default();
        new.server = server.into();
        SmtpTransportBuilder { info: new }
    }
}

/// Contains client configuration
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct SmtpTransportBuilder {
    info: SmtpInfo,
}

/// Builder for the SMTP `SmtpTransport`
impl SmtpTransportBuilder {
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

    /// Set the timeout duration
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.info.timeout = timeout;
        self
    }

    /// Set the port to use
    pub fn port(mut self, port: u16) -> Self {
        self.info.port = port;
        self
    }

    /// Set the TLS settings to use
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    pub fn tls(mut self, tls: Tls) -> Self {
        self.info.tls = tls;
        self
    }

    /// Build the client
    fn build_client(self) -> SmtpClient {
        SmtpClient { info: self.info }
    }

    /// Build the transport with custom pool settings
    #[cfg(feature = "r2d2")]
    pub fn build_with_pool(self, pool: Builder<SmtpClient>) -> SmtpTransport {
        let pool = pool.build_unchecked(self.build_client());
        SmtpTransport { inner: pool }
    }

    /// Build the transport
    ///
    /// If the `r2d2` feature is enabled an `Arc` wrapped pool is be created.
    /// Defaults:
    ///
    /// * 60 seconds idle timeout
    /// * 30 minutes max connection lifetime
    /// * max pool size of 10 connections
    pub fn build(self) -> SmtpTransport {
        let client = self.build_client();
        SmtpTransport {
            #[cfg(feature = "r2d2")]
            inner: Pool::builder()
                .min_idle(Some(0))
                .idle_timeout(Some(Duration::from_secs(60)))
                .build_unchecked(client),
            #[cfg(not(feature = "r2d2"))]
            inner: client,
        }
    }
}

/// Build client
#[derive(Clone)]
pub struct SmtpClient {
    info: SmtpInfo,
}

impl SmtpClient {
    /// Creates a new connection directly usable to send emails
    ///
    /// Handles encryption and authentication
    pub fn connection(&self) -> Result<SmtpConnection, Error> {
        #[allow(clippy::match_single_binding)]
        let tls_parameters = match self.info.tls {
            #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
            Tls::Wrapper(ref tls_parameters) => Some(tls_parameters),
            _ => None,
        };

        let mut conn = SmtpConnection::connect::<(&str, u16)>(
            (self.info.server.as_ref(), self.info.port),
            self.info.timeout,
            &self.info.hello_name,
            tls_parameters,
        )?;

        #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
        match self.info.tls {
            Tls::Opportunistic(ref tls_parameters) => {
                if conn.can_starttls() {
                    conn.starttls(tls_parameters, &self.info.hello_name)?;
                }
            }
            Tls::Required(ref tls_parameters) => {
                conn.starttls(tls_parameters, &self.info.hello_name)?;
            }
            _ => (),
        }

        if let Some(credentials) = &self.info.credentials {
            conn.auth(&self.info.authentication, &credentials)?;
        }
        Ok(conn)
    }
}
