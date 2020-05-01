//! The SMTP transport sends emails using the SMTP protocol.
//!
//! This SMTP client follows [RFC
//! 5321](https://tools.ietf.org/html/rfc5321), and is designed to efficiently send emails from an
//! application to a relay email server, as it relies as much as possible on the relay server
//! for sanity and RFC compliance checks.
//!
//! It implements the following extensions:
//!
//! * 8BITMIME ([RFC 6152](https://tools.ietf.org/html/rfc6152))
//! * AUTH ([RFC 4954](http://tools.ietf.org/html/rfc4954)) with PLAIN, LOGIN and XOAUTH2 mechanisms
//! * STARTTLS ([RFC 2487](http://tools.ietf.org/html/rfc2487))
//!

#[cfg(feature = "r2d2")]
use crate::transport::smtp::r2d2::SmtpConnectionManager;
use crate::Envelope;
use crate::{
    transport::smtp::{
        authentication::{Credentials, Mechanism, DEFAULT_MECHANISMS},
        client::{net::TlsParameters, SmtpConnection},
        commands::*,
        error::{Error, SmtpResult},
        extension::{ClientId, Extension, MailBodyParameter, MailParameter},
    },
    Transport,
};
#[cfg(feature = "native-tls")]
use native_tls::{Protocol, TlsConnector};
#[cfg(feature = "r2d2")]
use r2d2::Pool;
#[cfg(feature = "rustls")]
use rustls::ClientConfig;
use std::time::Duration;

#[cfg(feature = "rustls")]
use webpki_roots::TLS_SERVER_ROOTS;

pub mod authentication;
pub mod client;
pub mod commands;
pub mod error;
pub mod extension;
#[cfg(feature = "connection-pool")]
pub mod r2d2;
pub mod response;
pub mod util;

// Registered port numbers:
// https://www.iana.
// org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml

/// Default smtp port
pub const SMTP_PORT: u16 = 25;
/// Default submission port
pub const SUBMISSION_PORT: u16 = 587;
/// Default submission over TLS port
pub const SUBMISSIONS_PORT: u16 = 465;

/// Accepted protocols by default.
/// This removes TLS 1.0 and 1.1 compared to tls-native defaults.
// This is also rustls' default behavior
#[cfg(feature = "native-tls")]
const DEFAULT_TLS_MIN_PROTOCOL: Protocol = Protocol::Tlsv12;

/// How to apply TLS to a client connection
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub enum Tls {
    /// Insecure connection only (for testing purposes)
    None,
    /// Start with insecure connection and use `STARTTLS` when available
    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    Opportunistic(TlsParameters),
    /// Start with insecure connection and require `STARTTLS`
    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    Required(TlsParameters),
    /// Use TLS wrapped connection
    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    Wrapper(TlsParameters),
}

/// Contains client configuration
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct SmtpTransport {
    /// Name sent during EHLO
    hello_name: ClientId,
    /// Server we are connecting to
    server: String,
    /// Port to connect to
    port: u16,
    /// TLS security configuration
    tls: Tls,
    /// Optional enforced authentication mechanism
    authentication: Vec<Mechanism>,
    /// Credentials
    credentials: Option<Credentials>,
    /// Define network timeout
    /// It can be changed later for specific needs (like a different timeout for each SMTP command)
    timeout: Option<Duration>,
    /// Connection pool
    #[cfg(feature = "r2d2")]
    pool: Option<Pool>,
}

macro_rules! try_smtp (
    ($err: expr, $client: ident) => ({
        match $err {
            Ok(val) => val,
            Err(err) => {
                $client.abort();
                return Err(From::from(err))
            },
        }
    })
);

/// Builder for the SMTP `SmtpTransport`
impl SmtpTransport {
    /// Creates a new SMTP client
    ///
    /// Defaults are:
    ///
    /// * No authentication
    /// * A 60 seconds timeout for smtp commands
    /// * Port 587
    ///
    /// Consider using [`SmtpTransport::new`] instead, if possible.
    pub fn new<T: Into<String>>(server: T) -> Self {
        Self {
            server: server.into(),
            port: SUBMISSION_PORT,
            hello_name: ClientId::hostname(),
            credentials: None,
            authentication: DEFAULT_MECHANISMS.into(),
            timeout: Some(Duration::new(60, 0)),
            tls: Tls::None,
            #[cfg(feature = "r2d2")]
            pool: None,
        }
    }

    /// Simple and secure transport, should be used when possible.
    /// Creates an encrypted transport over submissions port, using the provided domain
    /// to validate TLS certificates.
    pub fn relay(relay: &str) -> Result<Self, Error> {
        #[cfg(feature = "native-tls")]
        let mut tls_builder = TlsConnector::builder();
        #[cfg(feature = "native-tls")]
        tls_builder.min_protocol_version(Some(DEFAULT_TLS_MIN_PROTOCOL));
        #[cfg(feature = "native-tls")]
        let tls_parameters = TlsParameters::new(relay.to_string(), tls_builder.build().unwrap());

        #[cfg(feature = "rustls")]
        let mut tls = ClientConfig::new();
        #[cfg(feature = "rustls")]
        tls.config
            .root_store
            .add_server_trust_anchors(&TLS_SERVER_ROOTS);
        #[cfg(feature = "rustls")]
        let tls_parameters = TlsParameters::new(relay.to_string(), tls);

        let new = Self::new(relay)
            .port(SUBMISSIONS_PORT)
            .tls(Tls::Wrapper(tls_parameters));

        #[cfg(feature = "r2d2")]
        // Pool with default configuration
        let new = new.pool(Pool::new(SmtpConnectionManager))?;

        Ok(new)
    }

    /// Creates a new local SMTP client to port 25
    ///
    /// Shortcut for local unencrypted relay (typical local email daemon that will handle relaying)
    pub fn unencrypted_localhost() -> Self {
        Self::new("localhost").port(SMTP_PORT)
    }

    /// Set the name used during EHLO
    pub fn hello_name(mut self, name: ClientId) -> Self {
        self.hello_name = name;
        self
    }

    /// Set the authentication mechanism to use
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Set the authentication mechanism to use
    pub fn authentication(mut self, mechanisms: Vec<Mechanism>) -> Self {
        self.authentication = mechanisms;
        self
    }

    /// Set the timeout duration
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the port to use
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the TLS settings to use
    pub fn tls(mut self, tls: Tls) -> Self {
        self.tls = tls;
        self
    }

    /// Set the TLS settings to use
    #[cfg(feature = "r2d2")]
    pub fn pool(mut self, pool: Pool) -> Self {
        self.pool = pool;
        self
    }

    /// Creates a new connection directly usable to send emails
    ///
    /// Handles encryption and authentication
    fn connection(&self) -> Result<SmtpConnection, Error> {
        let mut conn = SmtpConnection::connect::<(&str, u16)>(
            (self.server.as_ref(), self.port),
            self.timeout,
            &self.hello_name,
            match self.tls {
                Tls::Wrapper(ref tls_parameters) => Some(tls_parameters),
                _ => None,
            },
        )?;

        match self.tls {
            #[cfg(any(feature = "native-tls", feature = "rustls"))]
            Tls::Opportunistic(ref tls_parameters) => {
                if conn.can_starttls() {
                    try_smtp!(conn.starttls(tls_parameters, &self.hello_name), conn);
                }
            }
            #[cfg(any(feature = "native-tls", feature = "rustls"))]
            Tls::Required(ref tls_parameters) => {
                try_smtp!(conn.starttls(tls_parameters, &self.hello_name), conn);
            }
            _ => (),
        }

        match &self.credentials {
            Some(credentials) => {
                try_smtp!(
                    conn.auth(self.authentication.as_slice(), &credentials),
                    conn
                );
            }
            None => (),
        }

        Ok(conn)
    }
}

impl<'a> Transport<'a> for SmtpTransport {
    type Result = SmtpResult;

    /// Sends an email
    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Self::Result {
        #[cfg(feature = "r2d2")]
        let mut conn = match self.pool {
            Some(p) => p.get()?,
            None => self.connection()?,
        };
        #[cfg(not(feature = "r2d2"))]
        let mut conn = self.connection()?;

        // Mail
        let mut mail_options = vec![];

        if conn.server_info().supports_feature(Extension::EightBitMime) {
            mail_options.push(MailParameter::Body(MailBodyParameter::EightBitMime));
        }
        try_smtp!(
            conn.command(Mail::new(envelope.from().cloned(), mail_options,)),
            conn
        );

        // Recipient
        for to_address in envelope.to() {
            try_smtp!(conn.command(Rcpt::new(to_address.clone(), vec![])), conn);
        }

        // Data
        try_smtp!(conn.command(Data), conn);

        // Message content
        let result = try_smtp!(conn.message(email), conn);

        #[cfg(feature = "r2d2")]
        {
            if self.pool.is_none() {
                try_smtp!(conn.command(Quit), conn);
            }
        }
        #[cfg(not(feature = "r2d2"))]
        try_smtp!(conn.command(Quit), conn);

        Ok(result)
    }
}
