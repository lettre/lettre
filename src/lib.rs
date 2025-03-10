//! Lettre is an email library that allows creating and sending messages. It provides:
//!
//! * An easy to use email builder
//! * Pluggable email transports
//! * Unicode support
//! * Secure defaults
//! * Async support
//!
//! Lettre requires Rust 1.74 or newer.
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
//! * **pool** 📫: Connection pool for SMTP transport
//! * **hostname** 📫: Try to use the actual system hostname for the SMTP `CLIENTID`
//!
//! #### SMTP over TLS via the native-tls crate
//!
//! _Secure SMTP connections using TLS from the `native-tls` crate_
//!
//! Uses schannel on Windows, Security-Framework on macOS, and OpenSSL
//! on all other platforms.
//!
//! * **native-tls** 📫: TLS support for the synchronous version of the API
//! * **tokio1-native-tls**: TLS support for the `tokio1` async version of the API
//!
//! NOTE: native-tls isn't supported with `async-std`
//!
//! ##### Building lettre with OpenSSL
//!
//! When building lettre with native-tls on a system that makes
//! use of OpenSSL, the following packages will need to be installed
//! in order for the build and the compiled program to run properly.
//!
//! | Distro       | Build-time packages        | Runtime packages             |
//! | ------------ | -------------------------- | ---------------------------- |
//! | Debian       | `pkg-config`, `libssl-dev` | `libssl3`, `ca-certificates` |
//! | Alpine Linux | `pkgconf`, `openssl-dev`   | `libssl3`, `ca-certificates` |
//!
//! #### SMTP over TLS via the boring crate (Boring TLS)
//!
//! _Secure SMTP connections using TLS from the `boring-tls` crate_
//!
//! * **boring-tls**: TLS support for the synchronous version of the API
//! * **tokio1-boring-tls**: TLS support for the `tokio1` async version of the API
//!
//! NOTE: boring-tls isn't supported with `async-std`
//!
//! #### SMTP over TLS via the rustls crate
//!
//! _Secure SMTP connections using TLS from the `rustls` crate_
//!
//! * **rustls**: TLS support for the synchronous version of the API
//! * **tokio1-rustls**: TLS support for the `tokio1` async version of the API
//! * **async-std1-rustls**: TLS support for the `async-std1` async version of the API
//!
//! ##### rustls crypto backends
//!
//! _The crypto implementation to use with rustls_
//!
//! When the `rustls` feature is enabled, one of the following crypto backends MUST also
//! be enabled.
//!
//! * **aws-lc-rs**: use [AWS-LC] (via [`aws-lc-rs`]) as the `rustls` crypto backend
//! * **ring**: use [`ring`] as the `rustls` crypto backend
//!
//! When enabling `aws-lc-rs`, the `fips` feature can also be enabled to have
//! rustls use the FIPS certified module of AWS-LC.
//!
//! `aws-lc-rs` may require cmake on some platforms to compile.
//! `fips` always requires cmake and the Go compiler to compile.
//!
//! ##### rustls certificate verification backend
//!
//! _The TLS certificate verification backend to use with rustls_
//!
//! When the `rustls` feature is enabled, one of the following verification backends
//! MUST also be enabled.
//!
//! * **rustls-native-certs**: verify TLS certificates using the platform's native certificate store (see [`rustls-native-certs`])
//! * **webpki-roots**: verify TLS certificates against Mozilla's root certificates (see [`webpki-roots`])
//!
//! For the `rustls-native-certs` backend to work correctly, the following packages
//! will need to be installed in order for the build stage and the compiled program
//! to run properly.
//!
//! | Distro       | Build-time packages        | Runtime packages             |
//! | ------------ | -------------------------- | ---------------------------- |
//! | Debian       | none                       | `ca-certificates`            |
//! | Alpine Linux | none                       | `ca-certificates`            |
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
//! * **mime03**: Allow creating a [`ContentType`] from an existing [mime 0.3] `Mime` struct
//! * **dkim**: Add support for signing email with DKIM
//! * **web**: WebAssembly support using the `web-time` crate for time operations
//!
//! [`SMTP`]: crate::transport::smtp
//! [`sendmail`]: crate::transport::sendmail
//! [`file`]: crate::transport::file
//! [`ContentType`]: crate::message::header::ContentType
//! [tokio]: https://docs.rs/tokio/1
//! [async-std]: https://docs.rs/async-std/1
//! [AWS-LC]: https://github.com/aws/aws-lc
//! [`aws-lc-rs`]: https://crates.io/crates/aws-lc-rs
//! [`ring`]: https://crates.io/crates/ring
//! [`rustls-native-certs`]: https://crates.io/crates/rustls-native-certs
//! [`webpki-roots`]: https://crates.io/crates/webpki-roots
//! [Tokio 1.x]: https://docs.rs/tokio/1
//! [async-std 1.x]: https://docs.rs/async-std/1
//! [mime 0.3]: https://docs.rs/mime/0.3
//! [DKIM]: https://datatracker.ietf.org/doc/html/rfc6376

#![doc(html_root_url = "https://docs.rs/crate/lettre/0.11.15")]
#![doc(html_favicon_url = "https://lettre.rs/favicon.ico")]
#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/15113230?v=4")]
#![forbid(unsafe_code)]
#![deny(
    unreachable_pub,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    rust_2018_idioms,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::clone_on_ref_ptr,
    clippy::verbose_file_reads,
    clippy::unnecessary_self_imports,
    clippy::string_to_string,
    clippy::mem_forget,
    clippy::cast_lossless,
    clippy::inefficient_to_string,
    clippy::inline_always,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::manual_assert,
    clippy::unnecessary_join,
    clippy::wildcard_imports,
    clippy::str_to_string,
    clippy::empty_structs_with_brackets,
    clippy::zero_sized_map_values,
    clippy::manual_let_else,
    clippy::semicolon_if_nothing_returned,
    clippy::unnecessary_wraps,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::redundant_closure_for_method_calls,
    // Rust 1.86: clippy::unnecessary_semicolon,
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(lettre_ignore_tls_mismatch))]
mod compiletime_checks {
    #[cfg(all(feature = "rustls", not(feature = "aws-lc-rs"), not(feature = "ring")))]
    compile_error!(
        "feature `rustls` also requires either the `aws-lc-rs` or the `ring` feature to
    be enabled"
    );

    #[cfg(all(
        feature = "rustls",
        not(feature = "rustls-native-certs"),
        not(feature = "webpki-roots")
    ))]
    compile_error!(
        "feature `rustls` also requires either the `rustls-native-certs` or the `webpki-roots` feature to
    be enabled"
    );

    #[cfg(all(feature = "native-tls", feature = "boring-tls"))]
    compile_error!("feature \"native-tls\" and feature \"boring-tls\" cannot be enabled at the same time, otherwise
    the executable will fail to link.");

    #[cfg(all(
        feature = "tokio1",
        feature = "native-tls",
        not(feature = "tokio1-native-tls")
    ))]
    compile_error!("Lettre is being built with the `tokio1` and the `native-tls` features, but the `tokio1-native-tls` feature hasn't been turned on.
    If you were trying to opt into `rustls` and did not activate `native-tls`, disable the default-features of lettre in `Cargo.toml` and manually add the required features.
    Make sure to apply the same to any of your crate dependencies that use the `lettre` crate.");

    #[cfg(all(feature = "tokio1", feature = "rustls", not(feature = "tokio1-rustls")))]
    compile_error!("Lettre is being built with the `tokio1` and the `rustls` features, but the `tokio1-rustls` feature hasn't been turned on.
    If you'd like to use `native-tls` make sure that the `rustls` feature hasn't been enabled by mistake.
    Make sure to apply the same to any of your crate dependencies that use the `lettre` crate.");

    #[cfg(all(
        feature = "tokio1",
        feature = "boring-tls",
        not(feature = "tokio1-boring-tls")
    ))]
    compile_error!("Lettre is being built with the `tokio1` and the `boring-tls` features, but the `tokio1-boring-tls` feature hasn't been turned on.
    If you'd like to use `boring-tls` make sure that the `rustls` feature hasn't been enabled by mistake.
    Make sure to apply the same to any of your crate dependencies that use the `lettre` crate.");

    #[cfg(all(feature = "async-std1", feature = "native-tls"))]
    compile_error!("Lettre is being built with the `async-std1` and the `native-tls` features, but the async-std integration doesn't support native-tls yet.
If you'd like to work on the issue please take a look at https://github.com/lettre/lettre/issues/576.
If you were trying to opt into `rustls` and did not activate `native-tls`, disable the default-features of lettre in `Cargo.toml` and manually add the required features.
Make sure to apply the same to any of your crate dependencies that use the `lettre` crate.");

    #[cfg(all(
        feature = "async-std1",
        feature = "rustls",
        not(feature = "async-std1-rustls")
    ))]
    compile_error!("Lettre is being built with the `async-std1` and the `rustls` features, but the `async-std1-rustls` feature hasn't been turned on.
If you'd like to use `native-tls` make sure that the `rustls` hasn't been enabled by mistake.
Make sure to apply the same to any of your crate dependencies that use the `lettre` crate.");
}

pub mod address;
#[cfg(any(feature = "smtp-transport", feature = "dkim"))]
mod base64;
pub mod error;
#[cfg(any(feature = "tokio1", feature = "async-std1"))]
mod executor;
#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
pub mod message;
#[cfg(feature = "rustls")]
mod rustls_crypto;
mod time;
pub mod transport;

use std::error::Error as StdError;

#[cfg(feature = "async-std1")]
pub use self::executor::AsyncStd1Executor;
#[cfg(any(feature = "tokio1", feature = "async-std1"))]
pub use self::executor::Executor;
#[cfg(feature = "tokio1")]
pub use self::executor::Tokio1Executor;
#[cfg(any(feature = "tokio1", feature = "async-std1"))]
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
#[cfg(feature = "smtp-transport")]
pub use crate::transport::smtp::SmtpTransport;
#[doc(inline)]
pub use crate::transport::Transport;
use crate::{address::Envelope, error::Error};

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

#[cfg(test)]
#[cfg(feature = "builder")]
mod test {
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
