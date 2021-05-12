//! Lettre is an email library that allows creating and sending messages. It provides:
//!
//! * An easy to use email builder
//! * Pluggable email transports
//! * Unicode support
//! * Secure defaults
//! * Async support
//!
//! Lettre requires Rust 1.46 or newer.
//!
//! ## Features
//!
//! This section lists each lettre feature and briefly explains it.
//! More info about each module can be found in the corresponding module page.
//!
//! Features with `📫` near them are enabled by default.
//!
//! ### Typed message builder
//!
//! _Strongly typed [`message`] builder_
//!
//! * **builder** 📫: Enable the [`Message`] builder
//! * **hostname** 📫: Try to use the actual system hostname in the `Message-ID` header
//!
//! ### SMTP transport
//!
//! _Send emails using [`SMTP`]_
//!
//! * **smtp-transport** 📫: Enable the SMTP transport
//! * **r2d2** 📫: Connection pool for SMTP transport
//! * **hostname** 📫: Try to use the actual system hostname for the SMTP `CLIENTID`
//!
//! #### SMTP over TLS via the native-tls crate
//!
//! _Secure SMTP connections using TLS from the `native-tls` crate_
//!
//! Uses schannel on Windows, Security-Framework on macOS, and OpenSSL on Linux.
//!
//! * **native-tls** 📫: TLS support for the synchronous version of the API
//! * **tokio1-native-tls**: TLS support for the `tokio1` async version of the API
//!
//! NOTE: native-tls isn't supported with `async-std`
//!
//! #### SMTP over TLS via the rustls crate
//!
//! _Secure SMTP connections using TLS from the `rustls-tls` crate_
//!
//! Rustls uses [ring] as the cryptography implementation. As a result, [not all Rust's targets are supported][ring-support].
//!
//! * **rustls-tls**: TLS support for the synchronous version of the API
//! * **tokio1-rustls-tls**: TLS support for the `tokio1` async version of the API
//! * **async-std1-rustls-tls**: TLS support for the `async-std1` async version of the API
//!
//! ### Sendmail transport
//!
//! _Send emails using the [`sendmail`] command_
//!
//! * **sendmail-transport**: Enable the `sendmail` transport
//!
//! ### File transport
//!
//! _Save emails as an `.eml` [`file`]_
//!
//! * **file-transport**: Enable the file transport (saves emails into an `.eml` file)
//! * **file-transport-envelope**: Allow writing the envelope into a JSON file (additionally saves envelopes into a `.json` file)
//!
//! ### Async execution runtimes
//!
//! _Use [tokio] or [async-std] as an async execution runtime for sending emails_
//!
//! The correct runtime version must be chosen in order for lettre to work correctly.
//! For example, when sending emails from a Tokio 1.x context, the Tokio 1.x executor
//! ([`Tokio1Executor`]) must be used. Using a different version (for example Tokio 0.2.x),
//! or async-std, would result in a runtime panic.
//!
//! * **tokio1**: Allow to asynchronously send emails using [Tokio 1.x]
//! * **async-std1**: Allow to asynchronously send emails using [async-std 1.x]
//!
//! NOTE: native-tls isn't supported with `async-std`
//!
//! ### Misc features
//!
//! _Additional features_
//!
//! * **serde**: Serialization/Deserialization of entities
//! * **tracing**: Logging using the `tracing` crate
//!
//! [`SMTP`]: crate::transport::smtp
//! [`sendmail`]: crate::transport::sendmail
//! [`file`]: crate::transport::file
//! [tokio]: https://docs.rs/tokio/1
//! [async-std]: https://docs.rs/async-std/1
//! [ring]: https://github.com/briansmith/ring#ring
//! [ring-support]: https://github.com/briansmith/ring#online-automated-testing
//! [Tokio 1.x]: https://docs.rs/tokio/1
//! [async-std 1.x]: https://docs.rs/async-std/1

#![doc(html_root_url = "https://docs.rs/crate/lettre/0.10.0-beta.4")]
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
#[cfg(any(feature = "tokio1", feature = "async-std1"))]
mod executor;
#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
pub mod message;
pub mod transport;

#[cfg(feature = "async-std1")]
pub use self::executor::AsyncStd1Executor;
#[cfg(all(any(feature = "tokio1", feature = "async-std1")))]
pub use self::executor::Executor;
#[cfg(feature = "tokio1")]
pub use self::executor::Tokio1Executor;
#[cfg(all(any(feature = "tokio1", feature = "async-std1")))]
#[doc(inline)]
pub use self::transport::AsyncTransport;
pub use crate::address::Address;
#[cfg(feature = "builder")]
#[doc(inline)]
pub use crate::message::Message;
#[cfg(all(
    feature = "file-transport",
    any(feature = "tokio1", feature = "async-std1")
))]
#[doc(inline)]
pub use crate::transport::file::AsyncFileTransport;
#[cfg(feature = "file-transport")]
#[doc(inline)]
pub use crate::transport::file::FileTransport;
#[cfg(all(
    feature = "sendmail-transport",
    any(feature = "tokio1", feature = "async-std1")
))]
#[doc(inline)]
pub use crate::transport::sendmail::AsyncSendmailTransport;
#[cfg(feature = "sendmail-transport")]
#[doc(inline)]
pub use crate::transport::sendmail::SendmailTransport;
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio1", feature = "async-std1")
))]
pub use crate::transport::smtp::AsyncSmtpTransport;
#[doc(inline)]
pub use crate::transport::Transport;
use crate::{address::Envelope, error::Error};

#[cfg(feature = "smtp-transport")]
pub use crate::transport::smtp::SmtpTransport;
use std::error::Error as StdError;

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

#[cfg(test)]
#[cfg(feature = "builder")]
mod test {
    use std::convert::TryFrom;

    use super::*;
    use crate::message::{header, header::Headers, Mailbox, Mailboxes};

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
        headers.set(header::From::from(from));
        headers.set(header::Sender::from(sender));
        headers.set(header::To::from(to));

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
        headers.set(header::From::from(from));
        headers.set(header::Sender::from(sender));

        assert!(Envelope::try_from(&headers).is_err(),);
    }
}
