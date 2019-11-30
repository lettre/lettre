//! Lettre is a mailer written in Rust. It provides a simple email builder and several transports.
//!
//! This mailer contains the available transports for your emails.
//!

#![doc(html_root_url = "https://docs.rs/lettre/0.9.2")]
#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

pub mod error;
#[cfg(feature = "file-transport")]
pub mod file;
#[cfg(feature = "sendmail-transport")]
pub mod sendmail;
#[cfg(feature = "smtp-transport")]
pub mod smtp;
pub mod stub;

use crate::error::EmailResult;
use crate::error::Error;
#[cfg(feature = "file-transport")]
pub use crate::file::FileTransport;
#[cfg(feature = "sendmail-transport")]
pub use crate::sendmail::SendmailTransport;
#[cfg(feature = "smtp-transport")]
pub use crate::smtp::client::net::ClientTlsParameters;
#[cfg(all(feature = "smtp-transport", feature = "connection-pool"))]
pub use crate::smtp::r2d2::SmtpConnectionManager;
#[cfg(feature = "smtp-transport")]
pub use crate::smtp::{ClientSecurity, SmtpClient, SmtpTransport};
use fast_chemail::is_valid_email;
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::str::FromStr;

/// Email address
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde-impls", derive(serde::Serialize, serde::Deserialize))]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(address: String) -> EmailResult<EmailAddress> {
        if !EmailAddress::is_valid(&address) {
            Err(Error::InvalidEmailAddress)?;
        }
        Ok(EmailAddress(address))
    }

    pub fn is_valid(addr: &str) -> bool {
        is_valid_email(addr) || addr.ends_with("localhost")
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl FromStr for EmailAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EmailAddress::new(s.to_string())
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<OsStr> for EmailAddress {
    fn as_ref(&self) -> &OsStr {
        &self.0.as_ref()
    }
}

/// Simple email envelope representation
///
/// We only accept mailboxes, and do not support source routes (as per RFC).
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde-impls", derive(serde::Serialize, serde::Deserialize))]
pub struct Envelope {
    /// The envelope recipients' addresses
    ///
    /// This can not be empty.
    forward_path: Vec<EmailAddress>,
    /// The envelope sender address
    reverse_path: Option<EmailAddress>,
}

impl Envelope {
    /// Creates a new envelope, which may fail if `to` is empty.
    pub fn new(from: Option<EmailAddress>, to: Vec<EmailAddress>) -> EmailResult<Envelope> {
        if to.is_empty() {
            Err(Error::MissingTo)?;
        }
        Ok(Envelope {
            forward_path: to,
            reverse_path: from,
        })
    }

    /// Destination addresses of the envelope
    pub fn to(&self) -> &[EmailAddress] {
        self.forward_path.as_slice()
    }

    /// Source address of the envelope
    pub fn from(&self) -> Option<&EmailAddress> {
        self.reverse_path.as_ref()
    }
}

pub enum Message {
    Reader(Box<dyn Read + Send>),
    Bytes(Cursor<Vec<u8>>),
}

impl Read for Message {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Message::Reader(ref mut rdr) => rdr.read(buf),
            Message::Bytes(ref mut rdr) => rdr.read(buf),
        }
    }
}

/// Sendable email structure
pub struct SendableEmail {
    envelope: Envelope,
    message_id: String,
    message: Message,
}

impl SendableEmail {
    pub fn new(envelope: Envelope, message_id: String, message: Vec<u8>) -> SendableEmail {
        SendableEmail {
            envelope,
            message_id,
            message: Message::Bytes(Cursor::new(message)),
        }
    }

    pub fn new_with_reader(
        envelope: Envelope,
        message_id: String,
        message: Box<dyn Read + Send>,
    ) -> SendableEmail {
        SendableEmail {
            envelope,
            message_id,
            message: Message::Reader(message),
        }
    }

    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }

    pub fn message_id(&self) -> &str {
        &self.message_id
    }

    pub fn message(self) -> Message {
        self.message
    }

    pub fn message_to_string(mut self) -> Result<String, io::Error> {
        let mut message_content = String::new();
        self.message.read_to_string(&mut message_content)?;
        Ok(message_content)
    }
}

/// Transport method for emails
pub trait Transport<'a> {
    /// Result type for the transport
    type Result;

    /// Sends the email
    fn send<E: Into<SendableEmail>>(&mut self, email: E) -> Self::Result;
}
