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
//! #### Simple example with authentication
//!
//! A good starting point for sending emails via SMTP relay is to
//! do the following:
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "builder", any(feature = "native-tls", feature = "rustls-tls")))]
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! use lettre::{
//!     message::header::ContentType,
//!     transport::smtp::authentication::{Credentials, Mechanism},
//!     Message, SmtpTransport, Transport,
//! };
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .header(ContentType::TEXT_PLAIN)
//!     .body(String::from("Be happy!"))?;
//!
//! // Create the SMTPS transport
//! let sender = SmtpTransport::relay("smtp.example.com")?
//!     // Add credentials for authentication
//!     .credentials(Credentials::new(
//!         "username".to_owned(),
//!         "password".to_owned(),
//!     ))
//!     // Optionally configure expected authentication mechanism
//!     .authentication(vec![Mechanism::Plain])
//!     .build();
//!
//! // Send the email via remote relay
//! sender.send(&email)?;
//! # Ok(())
//! # }
//! ```
//!
//! #### Shortening configuration
//!
//! It can be very repetitive to ask the user for every SMTP connection parameter.
//! In some cases this can be simplified by using a connection URI instead.
//!
//! For more information take a look at [`SmtpTransport::from_url`] or [`AsyncSmtpTransport::from_url`].
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "builder", any(feature = "native-tls", feature = "rustls-tls")))]
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! use lettre::{
//!     message::header::ContentType,
//!     transport::smtp::authentication::{Credentials, Mechanism},
//!     Message, SmtpTransport, Transport,
//! };
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .header(ContentType::TEXT_PLAIN)
//!     .body(String::from("Be happy!"))?;
//!
//! // Create the SMTPS transport
//! let sender = SmtpTransport::from_url("smtps://username:password@smtp.example.com")?.build();
//!
//! // Send the email via remote relay
//! sender.send(&email)?;
//! # Ok(())
//! # }
//! ```
//!
//! #### Advanced configuration with custom TLS settings
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "builder", any(feature = "native-tls", feature = "rustls-tls")))]
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! use std::fs;
//!
//! use lettre::{
//!     message::header::ContentType,
//!     transport::smtp::client::{Certificate, Tls, TlsParameters},
//!     Message, SmtpTransport, Transport,
//! };
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .header(ContentType::TEXT_PLAIN)
//!     .body(String::from("Be happy!"))?;
//!
//! // Custom TLS configuration - Use a self signed certificate
//! let cert = fs::read("self-signed.crt")?;
//! let cert = Certificate::from_pem(&cert)?;
//! let tls = TlsParameters::builder(/* TLS SNI value */ "smtp.example.com".to_owned())
//!     .add_root_certificate(cert)
//!     .build()?;
//!
//! // Create the SMTPS transport
//! let sender = SmtpTransport::relay("smtp.example.com")?
//!     .tls(Tls::Wrapper(tls))
//!     .build();
//!
//! // Send the email via remote relay
//! sender.send(&email)?;
//! # Ok(())
//! # }
//! ```
//!
//! #### Connection pooling
//!
//! [`SmtpTransport`] and [`AsyncSmtpTransport`] store connections in
//! a connection pool by default. This avoids connecting and disconnecting
//! from the relay server for every message the application tries to send. For the connection pool
//! to work the instance of the transport **must** be reused.
//! In a webserver context it may go about this:
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "builder", any(feature = "native-tls", feature = "rustls-tls")))]
//! # fn test() {
//! use lettre::{
//!     message::header::ContentType,
//!     transport::smtp::{authentication::Credentials, PoolConfig},
//!     Message, SmtpTransport, Transport,
//! };
//! #
//! # type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
//!
//! /// The global application state
//! #[derive(Debug)]
//! struct AppState {
//!     smtp: SmtpTransport,
//!     // ... other global application parameters
//! }
//!
//! impl AppState {
//!     pub fn new(smtp_url: &str) -> Result<Self> {
//!         let smtp = SmtpTransport::from_url(smtp_url)?.build();
//!         Ok(Self { smtp })
//!     }
//! }
//!
//! fn handle_request(app_state: &AppState) -> Result<String> {
//!     let email = Message::builder()
//!         .from("NoBody <nobody@domain.tld>".parse()?)
//!         .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!         .to("Hei <hei@domain.tld>".parse()?)
//!         .subject("Happy new year")
//!         .header(ContentType::TEXT_PLAIN)
//!         .body(String::from("Be happy!"))?;
//!
//!     // Send the email via remote relay
//!     app_state.smtp.send(&email)?;
//!
//!     Ok("The email has successfully been sent!".to_owned())
//! }
//! # }
//! ```

use std::time::Duration;

use client::Tls;

#[cfg(any(feature = "tokio1", feature = "async-std1"))]
pub use self::async_transport::{AsyncSmtpTransport, AsyncSmtpTransportBuilder};
#[cfg(feature = "pool")]
pub use self::pool::PoolConfig;
pub use self::{
    error::Error,
    transport::{SmtpTransport, SmtpTransportBuilder},
};
#[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
use crate::transport::smtp::client::TlsParameters;
use crate::transport::smtp::{
    authentication::{Credentials, Mechanism, DEFAULT_MECHANISMS},
    client::SmtpConnection,
    extension::ClientId,
    response::Response,
};

#[cfg(any(feature = "tokio1", feature = "async-std1"))]
mod async_transport;
pub mod authentication;
pub mod client;
pub mod commands;
mod connection_url;
mod error;
pub mod extension;
#[cfg(feature = "pool")]
mod pool;
pub mod response;
mod transport;
pub(super) mod util;

// Registered port numbers:
// https://www.iana.
// org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml

/// Default smtp port
pub const SMTP_PORT: u16 = 25;
/// Default submission port
pub const SUBMISSION_PORT: u16 = 587;
/// Default submission over TLS port
///
/// Defined in [RFC8314](https://tools.ietf.org/html/rfc8314)
pub const SUBMISSIONS_PORT: u16 = 465;

/// Default timeout
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
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
            server: "localhost".to_owned(),
            port: SMTP_PORT,
            hello_name: ClientId::default(),
            credentials: None,
            authentication: DEFAULT_MECHANISMS.into(),
            timeout: Some(DEFAULT_TIMEOUT),
            tls: Tls::None,
        }
    }
}
