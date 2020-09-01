//! Lettre is an email library that allows creating and sending messages. It provides:
//!
//! * An easy to use email builder
//! * Pluggable email transports
//! * Unicode support
//! * Secure defaults
//!
//! Lettre requires Rust 1.40 or newer.
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
//! * **async-std1**: Allow to asyncronously send emails using async-std 1.x (SMTP isn't supported yet)
//! * **r2d2**: Connection pool for SMTP transport
//! * **tracing**: Logging using the `tracing` crate
//! * **serde**: Serialization/Deserialization of entities
//! * **hostname**: Ability to try to use actual hostname in SMTP transaction

#![doc(html_root_url = "https://docs.rs/lettre/0.10.0")]
#![doc(html_favicon_url = "https://lettre.at/favicon.png")]
#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/15113230?v=4")]
#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    unsafe_code
)]

pub mod address;
pub mod error;
#[cfg(feature = "builder")]
pub mod message;
pub mod transport;

use crate::error::Error;
#[cfg(feature = "builder")]
pub use crate::message::{
    header::{self, Headers},
    EmailFormat, Mailbox, Mailboxes, Message,
};
#[cfg(feature = "file-transport")]
pub use crate::transport::file::FileTransport;
#[cfg(feature = "sendmail-transport")]
pub use crate::transport::sendmail::SendmailTransport;
#[cfg(all(feature = "smtp-transport", feature = "connection-pool"))]
pub use crate::transport::smtp::r2d2::SmtpConnectionManager;
#[cfg(feature = "smtp-transport")]
pub use crate::transport::smtp::SmtpTransport;
#[cfg(all(feature = "smtp-transport", feature = "tokio02"))]
pub use crate::transport::smtp::{AsyncSmtpTransport, Tokio02Connector};
pub use crate::{address::Address, transport::stub::StubTransport};
#[cfg(any(feature = "async-std1", feature = "tokio02"))]
use async_trait::async_trait;
#[cfg(feature = "builder")]
use std::convert::TryFrom;
use std::{error::Error as StdError, fmt};

/// Simple email envelope representation
///
/// We only accept mailboxes, and do not support source routes (as per RFC).
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Envelope {
    /// The envelope recipients' addresses
    ///
    /// This can not be empty.
    forward_path: Vec<Address>,
    /// The envelope sender address
    reverse_path: Option<Address>,
}

impl Envelope {
    /// Creates a new envelope, which may fail if `to` is empty.
    pub fn new(from: Option<Address>, to: Vec<Address>) -> Result<Envelope, Error> {
        if to.is_empty() {
            return Err(Error::MissingTo);
        }
        Ok(Envelope {
            forward_path: to,
            reverse_path: from,
        })
    }

    /// Destination addresses of the envelope
    pub fn to(&self) -> &[Address] {
        self.forward_path.as_slice()
    }

    /// Source address of the envelope
    pub fn from(&self) -> Option<&Address> {
        self.reverse_path.as_ref()
    }
}

impl TryFrom<&Headers> for Envelope {
    type Error = Error;

    fn try_from(headers: &Headers) -> Result<Self, Self::Error> {
        let from = match headers.get::<header::Sender>() {
            // If there is a Sender, use it
            Some(header::Sender(a)) => Some(a.email.clone()),
            // ... else try From
            None => match headers.get::<header::From>() {
                Some(header::From(a)) => {
                    let from: Vec<Mailbox> = a.clone().into();
                    if from.len() > 1 {
                        return Err(Error::TooManyFrom);
                    }
                    Some(from[0].email.clone())
                }
                None => None,
            },
        };

        fn add_addresses_from_mailboxes(
            addresses: &mut Vec<Address>,
            mailboxes: Option<&Mailboxes>,
        ) {
            if let Some(mailboxes) = mailboxes {
                for mailbox in mailboxes.iter() {
                    addresses.push(mailbox.email.clone());
                }
            }
        }
        let mut to = vec![];
        add_addresses_from_mailboxes(&mut to, headers.get::<header::To>().map(|h| &h.0));
        add_addresses_from_mailboxes(&mut to, headers.get::<header::Cc>().map(|h| &h.0));
        add_addresses_from_mailboxes(&mut to, headers.get::<header::Bcc>().map(|h| &h.0));

        Self::new(from, to)
    }
}

/// Blocking Transport method for emails
pub trait Transport {
    /// Result types for the transport
    type Ok: fmt::Debug;
    type Error: StdError;

    /// Sends the email
    #[cfg(feature = "builder")]
    fn send(&self, message: &Message) -> Result<Self::Ok, Self::Error> {
        let raw = message.formatted();
        self.send_raw(message.envelope(), &raw)
    }

    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error>;
}

/// async-std 1.x based Transport method for emails
#[cfg(feature = "async-std1")]
#[async_trait]
pub trait AsyncStd1Transport {
    /// Result types for the transport
    type Ok: fmt::Debug;
    type Error: StdError;

    /// Sends the email
    #[cfg(feature = "builder")]
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
#[async_trait]
pub trait Tokio02Transport {
    /// Result types for the transport
    type Ok: fmt::Debug;
    type Error: StdError;

    /// Sends the email
    #[cfg(feature = "builder")]
    // TODO take &Message
    async fn send(&self, message: Message) -> Result<Self::Ok, Self::Error> {
        let raw = message.formatted();
        let envelope = message.envelope();
        self.send_raw(&envelope, &raw).await
    }

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::message::{header, Mailbox, Mailboxes};
    use hyperx::header::Headers;

    #[test]
    fn envelope_from_headers() {
        let mut from = Mailboxes::new();
        from.push("kayo@example.com".parse().unwrap());
        let mut to = Mailboxes::new();
        to.push("amousset@example.com".parse().unwrap());

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
        let mut from = Mailboxes::new();
        from.push("kayo@example.com".parse().unwrap());
        let sender = Mailbox::new(None, "kayo2@example.com".parse().unwrap());
        let mut to = Mailboxes::new();
        to.push("amousset@example.com".parse().unwrap());

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
        let mut from = Mailboxes::new();
        from.push("kayo@example.com".parse().unwrap());
        let sender = Mailbox::new(None, "kayo2@example.com".parse().unwrap());

        let mut headers = Headers::new();
        headers.set(header::From(from));
        headers.set(header::Sender(sender));

        assert!(Envelope::try_from(&headers).is_err(),);
    }
}
