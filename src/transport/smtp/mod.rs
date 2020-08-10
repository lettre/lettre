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
//! #### SMTP Transport
//!
//! This transport uses the SMTP protocol to send emails over the network (locally or remotely).
//!
//! It is designed to be:
//!
//! * Secured: email are encrypted by default
//! * Modern: unicode support for email content and sender/recipient addresses when compatible
//! * Fast: supports connection reuse and pooling
//!
//! This client is designed to send emails to a relay server, and should *not* be used to send
//! emails directly to the destination.
//!
//! The relay server can be the local email server, a specific host or a third-party service.
//!
//! #### Simple example
//!
//! This is the most basic example of usage:
//!
//! ```rust,no_run
//! # #[cfg(feature = "smtp-transport")]
//! # {
//! use lettre::{Message, Transport, SmtpTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse().unwrap())
//!     .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
//!     .to("Hei <hei@domain.tld>".parse().unwrap())
//!     .subject("Happy new year")
//!     .body("Be happy!")
//!     .unwrap();
//!
//! // Create local transport on port 25
//! let sender = SmtpTransport::unencrypted_localhost();
//! // Send the email on local relay
//! let result = sender.send(&email);
//!
//! assert!(result.is_ok());
//! # }
//! ```
//!
//! #### Complete example
//!
//! ```todo
//! # #[cfg(feature = "smtp-transport")]
//! # {
//! use lettre::transport::smtp::authentication::{Credentials, Mechanism};
//! use lettre::{Email, Envelope, Transport, SmtpClient};
//! use lettre::transport::smtp::extension::ClientId;
//!
//! let email_1 = Email::new(
//!     Envelope::new(
//!         Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
//!         vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
//!     ).unwrap(),
//!     "id1".to_string(),
//!     "Hello world".to_string().into_bytes(),
//! );
//!
//! let email_2 = Email::new(
//!     Envelope::new(
//!         Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
//!         vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
//!     ).unwrap(),
//!     "id2".to_string(),
//!     "Hello world a second time".to_string().into_bytes(),
//! );
//!
//! // Connect to a remote server on a custom port
//! let mut mailer = SmtpClient::new_simple("server.tld").unwrap()
//!    // Set the name sent during EHLO/HELO, default is `localhost`
//!    .hello_name(ClientId::Domain("my.hostname.tld".to_string()))
//!    // Add credentials for authentication
//!    .credentials(Credentials::new("username".to_string(), "password".to_string()))
//!    // Enable SMTPUTF8 if the server supports it
//!    .smtp_utf8(true)
//!    // Configure expected authentication mechanism
//!    .authentication_mechanism(Mechanism::Plain)
//!    // Enable connection reuse
//!    .connection_reuse(ConnectionReuseParameters::ReuseUnlimited).transport();
//!
//! let result_1 = mailer.send(&email_1);
//! assert!(result_1.is_ok());
//!
//! // The second email will use the same connection
//! let result_2 = mailer.send(&email_2);
//! assert!(result_2.is_ok());
//!
//! // Explicitly close the SMTP transaction as we enabled connection reuse
//! mailer.close();
//! # }
//! ```
//!
//! You can specify custom TLS settings:
//!
//! ```todo
//! # #[cfg(feature = "native-tls")]
//! # {
//! use lettre::{
//!     ClientSecurity, ClientTlsParameters, EmailAddress, Envelope,
//!     Email, SmtpClient, Transport,
//! };
//! use lettre::transport::smtp::authentication::{Credentials, Mechanism};
//! use lettre::transport::smtp::ConnectionReuseParameters;
//! use native_tls::{Protocol, TlsConnector};
//!
//!     let email = Email::new(
//!         Envelope::new(
//!             Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
//!             vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
//!         ).unwrap(),
//!         "message_id".to_string(),
//!         "Hello world".to_string().into_bytes(),
//!     );
//!
//!     let mut tls_builder = TlsConnector::builder();
//!     tls_builder.min_protocol_version(Some(Protocol::Tlsv10));
//!     let tls_parameters =
//!         ClientTlsParameters::new(
//!             "smtp.example.com".to_string(),
//!             tls_builder.build().unwrap()
//!         );
//!
//!     let mut mailer = SmtpClient::new(
//!         ("smtp.example.com", 465), ClientSecurity::Wrapper(tls_parameters)
//!     ).unwrap()
//!         .authentication_mechanism(Mechanism::Login)
//!         .credentials(Credentials::new(
//!             "example_username".to_string(), "example_password".to_string()
//!         ))
//!         .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
//!         .transport();
//!
//!     let result = mailer.send(&email);
//!
//!     assert!(result.is_ok());
//!
//!     mailer.close();
//! # }
//! ```
//!
//! #### Lower level
//!
//! You can also send commands, here is a simple email transaction without
//! error handling:
//!
//! ```rust,no_run
//! # #[cfg(feature = "smtp-transport")]
//! # {
//! use lettre::transport::smtp::{SMTP_PORT, extension::ClientId, commands::*, client::SmtpConnection};
//!
//! let hello = ClientId::new("my_hostname".to_string());
//! let mut client = SmtpConnection::connect(&("localhost", SMTP_PORT), None, &hello, None).unwrap();
//! client.command(
//!         Mail::new(Some("user@example.com".parse().unwrap()), vec![])
//!     ).unwrap();
//! client.command(
//!         Rcpt::new("user@example.org".parse().unwrap(), vec![])
//!       ).unwrap();
//! client.command(Data).unwrap();
//! client.message("Test email".as_bytes()).unwrap();
//! client.command(Quit).unwrap();
//! # }
//! ```

use std::time::Duration;

pub use self::transport::{SmtpClient, SmtpTransport, SmtpTransportBuilder};
#[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
use crate::transport::smtp::client::TlsParameters;
use crate::transport::smtp::{
    authentication::{Credentials, Mechanism, DEFAULT_MECHANISMS},
    client::SmtpConnection,
    error::Error,
    extension::ClientId,
    response::Response,
};
use client::Tls;

pub mod authentication;
pub mod client;
pub mod commands;
pub mod error;
pub mod extension;
#[cfg(feature = "r2d2")]
pub mod pool;
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
            hello_name: ClientId::hostname(),
            credentials: None,
            authentication: DEFAULT_MECHANISMS.into(),
            timeout: Some(DEFAULT_TIMEOUT),
            tls: Tls::None,
        }
    }
}
