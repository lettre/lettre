//! Lettre is a mailer written in Rust. It provides a simple email builder and several transports.
//!
//! This mailer contains the available transports for your emails. To be sendable, the
//! emails have to implement `SendableEmail`.
//!

#![doc(html_root_url = "https://docs.rs/lettre/0.8.0")]
#![deny(missing_docs, unsafe_code, unstable_features, warnings)]

#[cfg(feature = "smtp-transport")]
extern crate base64;
#[cfg(feature = "smtp-transport")]
extern crate bufstream;
#[cfg(feature = "crammd5-auth")]
extern crate crypto;
#[cfg(feature = "crammd5-auth")]
extern crate hex;
#[cfg(feature = "smtp-transport")]
extern crate hostname;
#[macro_use]
extern crate log;
#[cfg(feature = "smtp-transport")]
extern crate native_tls;
#[cfg(feature = "smtp-transport")]
#[macro_use]
extern crate nom;
#[cfg(feature = "serde-impls")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "file-transport")]
extern crate serde_json;

#[cfg(feature = "smtp-transport")]
pub mod smtp;
#[cfg(feature = "sendmail-transport")]
pub mod sendmail;
pub mod stub;
#[cfg(feature = "file-transport")]
pub mod file;

#[cfg(feature = "file-transport")]
pub use file::FileEmailTransport;
#[cfg(feature = "sendmail-transport")]
pub use sendmail::SendmailTransport;
#[cfg(feature = "smtp-transport")]
pub use smtp::{ClientSecurity, SmtpTransport};
#[cfg(feature = "smtp-transport")]
pub use smtp::client::net::ClientTlsParameters;
use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::error::Error as StdError;
use std::str::FromStr;

/// Error type for email content
#[derive(Debug)]
pub enum Error {
    /// Missing from in envelope
    MissingFrom,
    /// Missing to in envelope
    MissingTo,
    /// Invalid email
    InvalidEmailAddress,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::MissingFrom => "missing source address, invalid envelope",
            Error::MissingTo => "missing destination address, invalid envelope",
            Error::InvalidEmailAddress => "invalid email address",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.write_str(self.description())
    }
}

/// Email result type
pub type EmailResult<T> = Result<T, Error>;

/// Email address
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Creates a new `EmailAddress`. For now it makes no validation.
    pub fn new(address: String) -> EmailResult<EmailAddress> {
        // TODO make some basic sanity checks
        Ok(EmailAddress(address))
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

/// Simple email envelope representation
///
/// We only accept mailboxes, and do not support source routes (as per RFC).
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
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
            return Err(Error::MissingTo);
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

    /// Creates a new builder
    pub fn builder() -> EnvelopeBuilder {
        EnvelopeBuilder::new()
    }
}

/// Simple email envelope representation
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct EnvelopeBuilder {
    /// The envelope recipients' addresses
    to: Vec<EmailAddress>,
    /// The envelope sender address
    from: Option<EmailAddress>,
}

impl EnvelopeBuilder {
    /// Constructs an envelope with no recipients and an empty sender
    pub fn new() -> Self {
        EnvelopeBuilder {
            to: vec![],
            from: None,
        }
    }

    /// Adds a recipient
    pub fn to<S: Into<EmailAddress>>(mut self, address: S) -> Self {
        self.add_to(address);
        self
    }

    /// Adds a recipient
    pub fn add_to<S: Into<EmailAddress>>(&mut self, address: S) {
        self.to.push(address.into());
    }

    /// Sets the sender
    pub fn from<S: Into<EmailAddress>>(mut self, address: S) -> Self {
        self.set_from(address);
        self
    }

    /// Sets the sender
    pub fn set_from<S: Into<EmailAddress>>(&mut self, address: S) {
        self.from = Some(address.into());
    }

    /// Build the envelope
    pub fn build(self) -> EmailResult<Envelope> {
        Envelope::new(self.from, self.to)
    }
}

/// Email sendable by an SMTP client
pub trait SendableEmail<'a, T: Read + 'a> {
    /// Envelope
    fn envelope(&self) -> Envelope;
    /// Message ID, used for logging
    fn message_id(&self) -> String;
    /// Message content
    fn message(&'a self) -> Box<T>;
}

/// Transport method for emails
pub trait EmailTransport<'a, U: Read + 'a, V> {
    /// Sends the email
    fn send<T: SendableEmail<'a, U> + 'a>(&mut self, email: &'a T) -> V;
}

/// Minimal email structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub struct SimpleSendableEmail {
    /// Envelope
    envelope: Envelope,
    /// Message ID
    message_id: String,
    /// Message content
    message: Vec<u8>,
}

impl SimpleSendableEmail {
    /// Returns a new email
    pub fn new(
        from_address: String,
        to_addresses: &[String],
        message_id: String,
        message: String,
    ) -> EmailResult<SimpleSendableEmail> {
        let to: Result<Vec<EmailAddress>, Error> = to_addresses
            .iter()
            .map(|x| EmailAddress::new(x.clone()))
            .collect();
        Ok(SimpleSendableEmail::new_with_envelope(
            Envelope::new(Some(EmailAddress::new(from_address)?), to?)?,
            message_id,
            message,
        ))
    }

    /// Returns a new email from a valid envelope
    pub fn new_with_envelope(
        envelope: Envelope,
        message_id: String,
        message: String,
    ) -> SimpleSendableEmail {
        SimpleSendableEmail {
            envelope,
            message_id,
            message: message.into_bytes(),
        }
    }
}

impl<'a> SendableEmail<'a, &'a [u8]> for SimpleSendableEmail {
    fn envelope(&self) -> Envelope {
        self.envelope.clone()
    }

    fn message_id(&self) -> String {
        self.message_id.clone()
    }

    fn message(&'a self) -> Box<&[u8]> {
        Box::new(self.message.as_slice())
    }
}
