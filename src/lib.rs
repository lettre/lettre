//! Lettre is an email library that allows creating and sending messages. It provides:
//!
//! * An easy to use email builder
//! * Pluggable email transports
//! * Unicode support
//! * Secure defaults
//!
//! Lettre requires Rust 1.45 or newer.
//!
//! ## Optional features
//!
//! * **builder**: Message builder
//! * **file-transport**: Transport that write messages into a file
//! * **smtp-transport**: Transport over SMTP
//! * **sendmail-transport**: Transport over SMTP
//! * **rustls-tls**: TLS support with the `rustls` crate
//! * **native-tls**: TLS support with the `native-tls` crate
//! * **tokio02**: Allow to asyncronously send emails using tokio 0.2.x
//! * **tokio02-rustls-tls**: Async TLS support with the `rustls` crate using tokio 0.2
//! * **tokio02-native-tls**: Async TLS support with the `native-tls` crate using tokio 0.2
//! * **tokio03**: Allow to asyncronously send emails using tokio 0.3.x
//! * **tokio03-rustls-tls**: Async TLS support with the `rustls` crate using tokio 0.3
//! * **tokio03-native-tls**: Async TLS support with the `native-tls` crate using tokio 0.3
//! * **async-std1**: Allow to asyncronously send emails using async-std 1.x (SMTP isn't supported yet)
//! * **r2d2**: Connection pool for SMTP transport
//! * **tracing**: Logging using the `tracing` crate
//! * **serde**: Serialization/Deserialization of entities
//! * **hostname**: Ability to try to use actual hostname in SMTP transaction

#![doc(html_root_url = "https://docs.rs/crate/lettre/0.10.0-alpha.3")]
#![doc(html_favicon_url = "https://lettre.rs/favicon.ico")]
#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/15113230?v=4")]
#![forbid(unsafe_code)]
#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    rust_2018_idioms
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod address;
pub mod error;
#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
pub mod message;
pub mod transport;

#[cfg(feature = "builder")]
#[macro_use]
extern crate hyperx;

pub use crate::address::Address;
use crate::address::Envelope;
use crate::error::Error;
#[cfg(feature = "builder")]
pub use crate::message::Message;
#[cfg(feature = "file-transport")]
pub use crate::transport::file::FileTransport;
#[cfg(feature = "sendmail-transport")]
pub use crate::transport::sendmail::SendmailTransport;
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio02", feature = "tokio03")
))]
pub use crate::transport::smtp::AsyncSmtpTransport;
#[cfg(feature = "smtp-transport")]
pub use crate::transport::smtp::SmtpTransport;
#[cfg(all(feature = "smtp-transport", feature = "tokio02"))]
pub use crate::transport::smtp::Tokio02Connector;
#[cfg(all(feature = "smtp-transport", feature = "tokio03"))]
pub use crate::transport::smtp::Tokio03Connector;
#[cfg(any(feature = "async-std1", feature = "tokio02", feature = "tokio03"))]
use async_trait::async_trait;

/// Blocking Transport method for emails
pub trait Transport {
    /// Response produced by the Transport
    type Ok;
    /// Error produced by the Transport
    type Error;

    /// Sends the email
    #[cfg(feature = "builder")]
    #[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
    fn send(&self, message: &Message) -> Result<Self::Ok, Self::Error> {
        let raw = message.formatted();
        self.send_raw(message.envelope(), &raw)
    }

    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error>;
}

/// async-std 1.x based Transport method for emails
#[cfg(feature = "async-std1")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std1")))]
#[async_trait]
pub trait AsyncStd1Transport {
    /// Response produced by the Transport
    type Ok;
    /// Error produced by the Transport
    type Error;

    /// Sends the email
    #[cfg(feature = "builder")]
    #[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
    // TODO take &Message
    async fn send(&self, message: Message) -> Result<Self::Ok, Self::Error> {
        let raw = message.formatted();
        let envelope = message.envelope();
        self.send_raw(&envelope, &raw).await
    }

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error>;
}

/// tokio 0.2.x based Transport method for emails
#[cfg(feature = "tokio02")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio02")))]
#[async_trait]
pub trait Tokio02Transport {
    /// Response produced by the Transport
    type Ok;
    /// Error produced by the Transport
    type Error;

    /// Sends the email
    #[cfg(feature = "builder")]
    #[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
    // TODO take &Message
    async fn send(&self, message: Message) -> Result<Self::Ok, Self::Error> {
        let raw = message.formatted();
        let envelope = message.envelope();
        self.send_raw(&envelope, &raw).await
    }

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error>;
}

/// tokio 0.3.x based Transport method for emails
#[cfg(feature = "tokio03")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio03")))]
#[async_trait]
pub trait Tokio03Transport {
    /// Response produced by the Transport
    type Ok;
    /// Error produced by the Transport
    type Error;

    /// Sends the email
    #[cfg(feature = "builder")]
    #[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
    // TODO take &Message
    async fn send(&self, message: Message) -> Result<Self::Ok, Self::Error> {
        let raw = message.formatted();
        let envelope = message.envelope();
        self.send_raw(&envelope, &raw).await
    }

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error>;
}

#[cfg(test)]
#[cfg(feature = "builder")]
mod test {
    use super::*;
    use crate::message::{header, Mailbox, Mailboxes};
    use hyperx::header::Headers;
    use std::convert::TryFrom;

    #[test]
    fn envelope_from_headers() {
        let from = Mailboxes::new().with("kayo@example.com".parse().unwrap());
        let to = Mailboxes::new().with("amousset@example.com".parse().unwrap());

        let mut headers = Headers::new();
        headers.set(header::From(from));
        headers.set(header::To(to));

        assert_eq!(
            Envelope::try_from(&headers).unwrap(),
            Envelope::new(
                Some(Address::new("kayo", "example.com").unwrap()),
                vec![Address::new("amousset", "example.com").unwrap()]
            )
            .unwrap()
        );
    }

    #[test]
    fn envelope_from_headers_sender() {
        let from = Mailboxes::new().with("kayo@example.com".parse().unwrap());
        let sender = Mailbox::new(None, "kayo2@example.com".parse().unwrap());
        let to = Mailboxes::new().with("amousset@example.com".parse().unwrap());

        let mut headers = Headers::new();
        headers.set(header::From(from));
        headers.set(header::Sender(sender));
        headers.set(header::To(to));

        assert_eq!(
            Envelope::try_from(&headers).unwrap(),
            Envelope::new(
                Some(Address::new("kayo2", "example.com").unwrap()),
                vec![Address::new("amousset", "example.com").unwrap()]
            )
            .unwrap()
        );
    }

    #[test]
    fn envelope_from_headers_no_to() {
        let from = Mailboxes::new().with("kayo@example.com".parse().unwrap());
        let sender = Mailbox::new(None, "kayo2@example.com".parse().unwrap());

        let mut headers = Headers::new();
        headers.set(header::From(from));
        headers.set(header::Sender(sender));

        assert!(Envelope::try_from(&headers).is_err(),);
    }
}
