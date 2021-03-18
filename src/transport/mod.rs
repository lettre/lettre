//! ## Transports for sending emails
//!
//! This module contains `Transport`s for sending emails. A `Transport` implements a high-level API
//! for sending emails. It automatically manages the underlying resources and doesn't require any
//! specific knowledge of email protocols in order to be used.
//!
//! ### Get started
//!
//! Sending emails from your programs requires using an email relay, as client libraries are not
//! designed to handle email delivery by themselves. Depending on your infrastructure, your relay
//! could be:
//!
//! * a service from your Cloud or hosting provider
//! * an email server ([MTA] for Mail Transfer Agent, like Postfix or Exchange), running either
//! locally on your servers or accessible over the network
//! * a dedicated external service, like Mailchimp, Mailgun, etc.
//!
//! In most cases, the best option is to:
//!
//! * Use the [`SMTP`] transport, with the [`relay`] builder (or one of its async counterparts)
//!   with your server's hostname. They provide modern and secure defaults.
//! * Use the [`credentials`] method of the builder to pass your credentials.
//!
//! These should be enough to safely cover most use cases.
//!
//! ### Available transports
//!
//! The following transports are available:
//!
//! | Module       | Protocol | Sync API              | Async API                  | Description                                             |
//! | ------------ | -------- | --------------------- | -------------------------- | ------------------------------------------------------- |
//! | [`smtp`]     | SMTP     | [`SmtpTransport`]     | [`AsyncSmtpTransport`]     | Uses the SMTP protocol to send emails to a relay server |
//! | [`sendmail`] | Sendmail | [`SendmailTransport`] | [`AsyncSendmailTransport`] | Uses the `sendmail` command to send emails              |
//! | [`file`]     | File     | [`FileTransport`]     | [`AsyncFileTransport`]     | Saves the email as an `.eml` file                       |
//! | [`stub`]     | Debug    | [`StubTransport`]     | [`StubTransport`]          | Drops the email - Useful for debugging                  |
//!
//! ## Building an email
//!
//! Emails can either be built though [`Message`], which is a typed API for constructing emails
//! (find out more about it by going over the [`message`][crate::message] module),
//! or via external means.
//!
//! [`Message`]s can be sent via [`Transport::send`] or [`AsyncTransport::send`], while messages
//! built without lettre's [`message`][crate::message] APIs can be sent via [`Transport::send_raw`]
//! or [`AsyncTransport::send_raw`].
//!
//! ## Brief example
//!
//! This example shows how to build an email and send it via an SMTP relay server.
//! It is in no way a complete example, but it shows how to get started with lettre.
//! More examples can be found by looking at the specific modules, linked in the _Module_ column
//! of the [table above](#transports-for-sending-emails).
//!
//! ```rust,no_run
//! # use std::error::Error;
//! #
//! # #[cfg(all(feature = "builder", feature = "smtp-transport"))]
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use lettre::transport::smtp::authentication::Credentials;
//! use lettre::{Message, SmtpTransport, Transport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body(String::from("Be happy!"))?;
//!
//! let creds = Credentials::new("smtp_username".to_string(), "smtp_password".to_string());
//!
//! // Open a remote connection to the SMTP relay server
//! let mailer = SmtpTransport::relay("smtp.gmail.com")?
//!     .credentials(creds)
//!     .build();
//!
//! // Send the email
//! match mailer.send(&email) {
//!     Ok(_) => println!("Email sent successfully!"),
//!     Err(e) => panic!("Could not send email: {:?}", e),
//! }
//! # Ok(())
//! # }
//! # #[cfg(not(all(feature = "builder", feature = "smtp-transport")))]
//! # fn main() {}
//! ```
//!
//! [MTA]: https://en.wikipedia.org/wiki/Message_transfer_agent
//! [`SMTP`]: crate::transport::smtp
//! [`relay`]: crate::SmtpTransport::relay
//! [`starttls_relay`]: crate::SmtpTransport::starttls_relay
//! [`credentials`]: crate::transport::smtp::SmtpTransportBuilder::credentials
//! [`Message`]: crate::Message
//! [`file`]: self::file
//! [`SmtpTransport`]: crate::SmtpTransport
//! [`AsyncSmtpTransport`]: crate::AsyncSmtpTransport
//! [`SendmailTransport`]: crate::SendmailTransport
//! [`AsyncSendmailTransport`]: crate::AsyncSendmailTransport
//! [`FileTransport`]: crate::FileTransport
//! [`AsyncFileTransport`]: crate::AsyncFileTransport
//! [`StubTransport`]: crate::transport::stub::StubTransport

#[cfg(any(feature = "async-std1", feature = "tokio02", feature = "tokio1"))]
use async_trait::async_trait;

use crate::Envelope;
#[cfg(feature = "builder")]
use crate::Message;

#[cfg(feature = "file-transport")]
#[cfg_attr(docsrs, doc(cfg(feature = "file-transport")))]
pub mod file;
#[cfg(feature = "sendmail-transport")]
#[cfg_attr(docsrs, doc(cfg(feature = "sendmail-transport")))]
pub mod sendmail;
#[cfg(feature = "smtp-transport")]
#[cfg_attr(docsrs, doc(cfg(feature = "smtp-transport")))]
pub mod smtp;
pub mod stub;

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

/// Async Transport method for emails
#[cfg(any(feature = "tokio02", feature = "tokio1", feature = "async-std1"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "tokio02", feature = "tokio1", feature = "async-std1")))
)]
#[async_trait]
pub trait AsyncTransport {
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
