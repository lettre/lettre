#[cfg(feature = "pool")]
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "pool")]
use super::pool::sync_impl::Pool;
#[cfg(feature = "pool")]
use super::PoolConfig;
use super::{ClientId, Credentials, Error, Mechanism, Response, SmtpConnection, SmtpInfo};
#[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
use super::{Tls, TlsParameters, SUBMISSIONS_PORT, SUBMISSION_PORT};
use crate::{address::Envelope, Transport};

/// Sends emails using the SMTP protocol
#[cfg_attr(docsrs, doc(cfg(feature = "smtp-transport")))]
#[derive(Clone)]
pub struct SmtpTransport {
    #[cfg(feature = "pool")]
    inner: Arc<Pool>,
    #[cfg(not(feature = "pool"))]
    inner: SmtpClient,
}

impl Transport for SmtpTransport {
    type Ok = Response;
    type Error = Error;

    /// Sends an email
    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut conn = self.inner.connection()?;

        let result = conn.send(envelope, email)?;

        #[cfg(not(feature = "pool"))]
        conn.quit()?;

        Ok(result)
    }
}

impl SmtpTransport {
    /// Simple and secure transport, using TLS connections to communicate with the SMTP server
    ///
    /// The right option for most SMTP servers.
    ///
    /// Creates an encrypted transport over submissions port, using the provided domain
    /// to validate TLS certificates.
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls")))
    )]
    pub fn relay(relay: &str) -> Result<SmtpTransportBuilder, Error> {
        let tls_parameters = TlsParameters::new(relay.into())?;

        Ok(Self::builder_dangerous(relay)
            .port(SUBMISSIONS_PORT)
            .tls(Tls::Wrapper(tls_parameters)))
    }

    /// Simple and secure transport, using STARTTLS to obtain encrypted connections
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
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls")))
    )]
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
    /// * A 60-seconds timeout for smtp commands
    /// * Port 25
    ///
    /// Consider using [`SmtpTransport::relay`](#method.relay) or
    /// [`SmtpTransport::starttls_relay`](#method.starttls_relay) instead,
    /// if possible.
    pub fn builder_dangerous<T: Into<String>>(server: T) -> SmtpTransportBuilder {
        SmtpTransportBuilder::new(server)
    }

    /// Creates a `SmtpTransportBuilder` from a connection URL
    ///
    /// The protocol, credentials, host and port can be provided in a single URL.
    /// Use the scheme `smtp` for an unencrypted relay, `smtps` for SMTP over TLS
    /// and `smtp` with the query parameter tls=required or tls=opportunistic for STARTTLS
    ///
    /// ```rust,no_run
    /// use lettre::{
    ///     message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    ///     SmtpTransport, Transport,
    /// };
    ///
    /// let email = Message::builder()
    ///     .from("NoBody <nobody@domain.tld>".parse().unwrap())
    ///     .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
    ///     .to("Hei <hei@domain.tld>".parse().unwrap())
    ///     .subject("Happy new year")
    ///     .header(ContentType::TEXT_PLAIN)
    ///     .body(String::from("Be happy!"))
    ///     .unwrap();
    ///
    /// // Open a remote connection to example
    /// let mailer = SmtpTransport::from_url("smtps://username:password@smtp.example.com:465")
    ///     .unwrap()
    ///     .build();
    ///
    /// // Send the email
    /// match mailer.send(&email) {
    ///     Ok(_) => println!("Email sent successfully!"),
    ///     Err(e) => panic!("Could not send email: {e:?}"),
    /// }
    /// ```
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls")))
    )]
    pub fn from_url(connection_url: &str) -> Result<SmtpTransportBuilder, Error> {
        super::connection_url::from_connection_url(connection_url)
    }

    /// Tests the SMTP connection
    ///
    /// `test_connection()` tests the connection by using the SMTP NOOP command.
    /// The connection is closed afterward if a connection pool is not used.
    pub fn test_connection(&self) -> Result<bool, Error> {
        let mut conn = self.inner.connection()?;

        let is_connected = conn.test_connected();

        #[cfg(not(feature = "pool"))]
        conn.quit()?;

        Ok(is_connected)
    }
}

/// Contains client configuration.
/// Instances of this struct can be created using functions of [`SmtpTransport`].
#[derive(Debug, Clone)]
pub struct SmtpTransportBuilder {
    info: SmtpInfo,
    #[cfg(feature = "pool")]
    pool_config: PoolConfig,
}

/// Builder for the SMTP `SmtpTransport`
impl SmtpTransportBuilder {
    // Create new builder with default parameters
    pub(crate) fn new<T: Into<String>>(server: T) -> Self {
        let new = SmtpInfo {
            server: server.into(),
            ..Default::default()
        };

        Self {
            info: new,
            #[cfg(feature = "pool")]
            pool_config: PoolConfig::default(),
        }
    }

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
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls")))
    )]
    pub fn tls(mut self, tls: Tls) -> Self {
        self.info.tls = tls;
        self
    }

    /// Use a custom configuration for the connection pool
    ///
    /// Defaults can be found at [`PoolConfig`]
    #[cfg(feature = "pool")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pool")))]
    pub fn pool_config(mut self, pool_config: PoolConfig) -> Self {
        self.pool_config = pool_config;
        self
    }

    /// Build the transport
    ///
    /// If the `pool` feature is enabled, an `Arc` wrapped pool is created.
    /// Defaults can be found at [`PoolConfig`]
    pub fn build(self) -> SmtpTransport {
        let client = SmtpClient { info: self.info };

        #[cfg(feature = "pool")]
        let client = Pool::new(self.pool_config, client);

        SmtpTransport { inner: client }
    }
}

/// Build client
#[derive(Debug, Clone)]
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
            #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
            Tls::Wrapper(ref tls_parameters) => Some(tls_parameters),
            _ => None,
        };

        #[allow(unused_mut)]
        let mut conn = SmtpConnection::connect::<(&str, u16)>(
            (self.info.server.as_ref(), self.info.port),
            self.info.timeout,
            &self.info.hello_name,
            tls_parameters,
            None,
        )?;

        #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
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
            conn.auth(&self.info.authentication, credentials)?;
        }
        Ok(conn)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        transport::smtp::{authentication::Credentials, client::Tls},
        SmtpTransport,
    };

    #[test]
    fn transport_from_url() {
        let builder = SmtpTransport::from_url("smtp://127.0.0.1:2525").unwrap();

        assert_eq!(builder.info.port, 2525);
        assert!(matches!(builder.info.tls, Tls::None));
        assert_eq!(builder.info.server, "127.0.0.1");

        let builder =
            SmtpTransport::from_url("smtps://username:password@smtp.example.com:465").unwrap();

        assert_eq!(builder.info.port, 465);
        assert_eq!(
            builder.info.credentials,
            Some(Credentials::new(
                "username".to_owned(),
                "password".to_owned()
            ))
        );
        assert!(matches!(builder.info.tls, Tls::Wrapper(_)));
        assert_eq!(builder.info.server, "smtp.example.com");

        let builder =
            SmtpTransport::from_url("smtp://username:password@smtp.example.com:587?tls=required")
                .unwrap();

        assert_eq!(builder.info.port, 587);
        assert_eq!(
            builder.info.credentials,
            Some(Credentials::new(
                "username".to_owned(),
                "password".to_owned()
            ))
        );
        assert!(matches!(builder.info.tls, Tls::Required(_)));

        let builder = SmtpTransport::from_url(
            "smtp://username:password@smtp.example.com:587?tls=opportunistic",
        )
        .unwrap();

        assert_eq!(builder.info.port, 587);
        assert!(matches!(builder.info.tls, Tls::Opportunistic(_)));

        let builder = SmtpTransport::from_url("smtps://smtp.example.com").unwrap();

        assert_eq!(builder.info.port, 465);
        assert_eq!(builder.info.credentials, None);
        assert!(matches!(builder.info.tls, Tls::Wrapper(_)));
    }
}
