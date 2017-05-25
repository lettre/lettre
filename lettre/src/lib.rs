//! Lettre is a mailer written in Rust. It provides a simple email builder and several transports.
//!
//! This mailer contains the available transports for your emails. To be sendable, the
//! emails have to implement `SendableEmail`.
//!

#![deny(missing_docs, unsafe_code, unstable_features, warnings, missing_debug_implementations)]

#[macro_use]
extern crate log;
extern crate base64;
extern crate hex;
extern crate crypto;
extern crate bufstream;
extern crate openssl;

pub mod smtp;
pub mod sendmail;
pub mod stub;
pub mod file;

/// Email sendable by an SMTP client
pub trait SendableEmail {
    /// To
    fn to(&self) -> Vec<String>;
    /// From
    fn from(&self) -> String;
    /// Message ID, used for logging
    fn message_id(&self) -> String;
    /// Message content
    fn message(self) -> String;
}

/// Transport method for emails
pub trait EmailTransport<U> {
    /// Sends the email
    fn send<T: SendableEmail>(&mut self, email: T) -> U;
    /// Close the transport explicitly
    fn close(&mut self);
}

/// Minimal email structure
#[derive(Debug,Clone)]
pub struct SimpleSendableEmail {
    /// To
    to: Vec<String>,
    /// From
    from: String,
    /// Message ID
    message_id: String,
    /// Message content
    message: String,
}

impl SimpleSendableEmail {
    /// Returns a new email
    pub fn new(from_address: &str,
               to_addresses: Vec<&str>,
               message_id: &str,
               message: &str)
               -> SimpleSendableEmail {
        SimpleSendableEmail {
            from: from_address.to_string(),
            to: to_addresses.iter().map(|s| s.to_string()).collect(),
            message_id: message_id.to_string(),
            message: message.to_string(),
        }
    }
}

impl SendableEmail for SimpleSendableEmail {
    fn to(&self) -> Vec<String> {
        self.to.clone()
    }

    fn from(&self) -> String {
        self.from.clone()
    }

    fn message_id(&self) -> String {
        self.message_id.clone()
    }

    fn message(self) -> String {
        self.message
    }
}
