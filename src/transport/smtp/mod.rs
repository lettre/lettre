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
//! * AUTH ([RFC 4954](https://tools.ietf.org/html/rfc4954)) with PLAIN, LOGIN and XOAUTH2 mechanisms
//! * STARTTLS ([RFC 2487](https://tools.ietf.org/html/rfc2487))
//!
//! #### SMTP Transport
//!
//! This transport uses the SMTP protocol to send emails over the network (locally or remotely).
//!
//! It is designed to be:
//!
//! * Secured: connections are encrypted by default
//! * Modern: unicode support for email contents and sender/recipient addresses when compatible
//! * Fast: supports connection reuse and pooling
//!
//! This client is designed to send emails to a relay server, and should *not* be used to send
//! emails directly to the destination server.
//!
//! The relay server can be the local email server, a specific host or a third-party service.
//!
//! #### Simple example
//!
//! This is the most basic example of usage:
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "builder", any(feature = "native-tls", feature = "rustls-tls")))]
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! use lettre::{Message, Transport, SmtpTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body(String::from("Be happy!"))?;
//!
//! // Create TLS transport on port 465
//! let sender = SmtpTransport::relay("smtp.example.com")?
//!     .build();
//! // Send the email via remote relay
//! let result = sender.send(&email);
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//! ```
//!
//! #### Authentication
//!
//! Example with authentication and connection pool:
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "builder", any(feature = "native-tls", feature = "rustls-tls")))]
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! use lettre::{Message, Transport, SmtpTransport, transport::smtp::{PoolConfig, authentication::{Credentials, Mechanism}}};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body(String::from("Be happy!"))?;
//!
//! // Create TLS transport on port 587 with STARTTLS
//! let sender = SmtpTransport::starttls_relay("smtp.example.com")?
//!     // Add credentials for authentication
//!     .credentials(Credentials::new("username".to_string(), "password".to_string()))
//!     // Configure expected authentication mechanism
//!     .authentication(vec![Mechanism::Plain])
//!     // Connection pool settings
//!     .pool_config( PoolConfig::new().max_size(20))
//!     .build();
//!
//! // Send the email via remote relay
//! let result = sender.send(&email);
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//! ```
//!
//! You can specify custom TLS settings:
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "builder", any(feature = "native-tls", feature = "rustls-tls")))]
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! use lettre::{Message, Transport, SmtpTransport, transport::smtp::client::{TlsParameters, Tls}};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body(String::from("Be happy!"))?;
//!
//! // Custom TLS configuration
//! let tls = TlsParameters::builder("smtp.example.com".to_string())
//!           .dangerous_accept_invalid_certs(true).build()?;
//!
//! // Create TLS transport on port 465
//! let sender = SmtpTransport::relay("smtp.example.com")?
//!     // Custom TLS configuration
//!     .tls(Tls::Required(tls))
//!     .build();
//!
//! // Send the email via remote relay
//! let result = sender.send(&email);
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "async-std1")]
pub use self::async_transport::AsyncStd1Connector;
#[cfg(feature = "tokio02")]
pub use self::async_transport::Tokio02Connector;
#[cfg(feature = "tokio1")]
pub use self::async_transport::Tokio1Connector;
#[cfg(any(feature = "tokio02", feature = "tokio1", feature = "async-std1"))]
pub use self::async_transport::{
    AsyncSmtpConnector, AsyncSmtpTransport, AsyncSmtpTransportBuilder,
};
#[cfg(feature = "r2d2")]
pub use self::pool::PoolConfig;
#[cfg(feature = "r2d2")]
pub(crate) use self::transport::SmtpClient;
pub use self::{
    error::Error,
    transport::{SmtpTransport, SmtpTransportBuilder},
};
#[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
use crate::transport::smtp::client::TlsParameters;
use crate::transport::smtp::{
    authentication::{Credentials, Mechanism, DEFAULT_MECHANISMS},
    client::SmtpConnection,
    extension::ClientId,
    response::Response,
};
use client::Tls;
use std::time::Duration;

#[cfg(any(feature = "tokio02", feature = "tokio1", feature = "async-std1"))]
mod async_transport;
pub mod authentication;
pub mod client;
pub mod commands;
mod error;
pub mod extension;
#[cfg(feature = "r2d2")]
mod pool;
pub mod response;
mod transport;
pub mod util;

// Registered port numbers:
// https://www.iana.
// org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml

/// Default smtp port
pub const SMTP_PORT: u16 = 25;
/// Default submission port
pub const SUBMISSION_PORT: u16 = 587;
/// Default submission over TLS port
///
/// https://tools.ietf.org/html/rfc8314
pub const SUBMISSIONS_PORT: u16 = 465;

/// Default timeout
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

#[allow(missing_debug_implementations)]
#[derive(Clone)]
struct SmtpInfo {
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
}

impl Default for SmtpInfo {
    fn default() -> Self {
        Self {
            server: "localhost".to_string(),
            port: SMTP_PORT,
            hello_name: ClientId::default(),
            credentials: None,
            authentication: DEFAULT_MECHANISMS.into(),
            timeout: Some(DEFAULT_TIMEOUT),
            tls: Tls::None,
        }
    }
}
