//! Lettre provides an email builder and several email transports.
//!

#![doc(html_root_url = "https://docs.rs/lettre/0.10.0")]
#![doc(html_favicon_url = "https://blog.lettre.at/favicon.ico")]
#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/15113230?v=4")]
#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
// FIXME remove unsafe?
//    unsafe_code,
    unstable_features,
    unused_import_braces
)]

pub mod address;
#[cfg(feature = "builder")]
pub mod builder;
pub mod error;
#[cfg(feature = "file-transport")]
pub mod file;
#[cfg(feature = "sendmail-transport")]
pub mod sendmail;
#[cfg(feature = "smtp-transport")]
pub mod smtp;
pub mod stub;

pub use crate::address::Address;
//#[cfg(feature = "builder")]
//pub use crate::builder::Message;
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
use std::io::{self, Cursor, Read};

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

// FIXME merge with emailmessage payload?
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
pub struct Email {
    envelope: Envelope,
    message_id: String,
    message: Message,
}

impl Email {
    /// Creates a new email builder
    //#[cfg(feature = "builder")]
    //pub fn builder() -> EmailBuilder {
    //    EmailBuilder::new()
    //}

    pub fn new(envelope: Envelope, message_id: String, message: Vec<u8>) -> Email {
        Email {
            envelope,
            message_id,
            message: Message::Bytes(Cursor::new(message)),
        }
    }

    pub fn new_with_reader(
        envelope: Envelope,
        message_id: String,
        message: Box<dyn Read + Send>,
    ) -> Email {
        Email {
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
    fn send<E: Into<Email>>(&mut self, email: E) -> Self::Result;
}
