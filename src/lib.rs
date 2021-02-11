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
//! * **file-transport-envelope**: Allow writing the envelope into a JSON file
//! * **smtp-transport**: Transport over SMTP
//! * **sendmail-transport**: Transport over SMTP
//! * **rustls-tls**: TLS support with the `rustls` crate
//! * **native-tls**: TLS support with the `native-tls` crate
//! * **tokio02**: Allow to asyncronously send emails using tokio 0.2.x
//! * **tokio02-rustls-tls**: Async TLS support with the `rustls` crate using tokio 0.2
//! * **tokio02-native-tls**: Async TLS support with the `native-tls` crate using tokio 0.2
//! * **tokio1**: Allow to asyncronously send emails using tokio 1.x
//! * **tokio1-rustls-tls**: Async TLS support with the `rustls` crate using tokio 1.x
//! * **tokio1-native-tls**: Async TLS support with the `native-tls` crate using tokio 1.x
//! * **async-std1**: Allow to asynchronously send emails using async-std 1.x
//! * NOTE: native-tls isn't supported with async-std at the moment
//! * **async-std1-rustls-tls**: Async TLS support with the `rustls` crate using async-std 1.x
//! * **r2d2**: Connection pool for SMTP transport
//! * **tracing**: Logging using the `tracing` crate
//! * **serde**: Serialization/Deserialization of entities
//! * **hostname**: Ability to try to use actual hostname in SMTP transaction

#![doc(html_root_url = "https://docs.rs/crate/lettre/0.10.0-alpha.5")]
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
#[cfg(all(any(feature = "tokio02", feature = "tokio1", feature = "async-std1")))]
mod executor;
#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
pub mod message;
pub mod transport;

#[cfg(feature = "builder")]
#[macro_use]
extern crate hyperx;

#[cfg(feature = "async-std1")]
pub use self::executor::AsyncStd1Executor;
#[cfg(all(any(feature = "tokio02", feature = "tokio1", feature = "async-std1")))]
pub use self::executor::Executor;
#[cfg(feature = "tokio02")]
pub use self::executor::Tokio02Executor;
#[cfg(feature = "tokio1")]
pub use self::executor::Tokio1Executor;
#[cfg(all(any(feature = "tokio02", feature = "tokio1", feature = "async-std1")))]
pub use self::transport::AsyncTransport;
pub use crate::address::Address;
#[cfg(feature = "builder")]
pub use crate::message::Message;
#[cfg(all(
    feature = "file-transport",
    any(feature = "tokio02", feature = "tokio1", feature = "async-std1")
))]
pub use crate::transport::file::AsyncFileTransport;
#[cfg(feature = "file-transport")]
pub use crate::transport::file::FileTransport;
#[cfg(all(
    feature = "sendmail-transport",
    any(feature = "tokio02", feature = "tokio1", feature = "async-std1")
))]
pub use crate::transport::sendmail::AsyncSendmailTransport;
#[cfg(feature = "sendmail-transport")]
pub use crate::transport::sendmail::SendmailTransport;
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio02", feature = "tokio1")
))]
pub use crate::transport::smtp::AsyncSmtpTransport;
pub use crate::transport::Transport;
use crate::{address::Envelope, error::Error};

#[doc(hidden)]
#[allow(deprecated)]
#[cfg(all(feature = "smtp-transport", feature = "async-std1"))]
pub use crate::transport::smtp::AsyncStd1Connector;
#[cfg(feature = "smtp-transport")]
pub use crate::transport::smtp::SmtpTransport;
#[doc(hidden)]
#[allow(deprecated)]
#[cfg(all(feature = "smtp-transport", feature = "tokio02"))]
pub use crate::transport::smtp::Tokio02Connector;
#[doc(hidden)]
#[allow(deprecated)]
#[cfg(all(feature = "smtp-transport", feature = "tokio1"))]
pub use crate::transport::smtp::Tokio1Connector;
#[doc(hidden)]
#[cfg(feature = "async-std1")]
pub use crate::transport::AsyncStd1Transport;
#[doc(hidden)]
#[cfg(feature = "tokio02")]
pub use crate::transport::Tokio02Transport;
#[doc(hidden)]
#[cfg(feature = "tokio1")]
pub use crate::transport::Tokio1Transport;

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
