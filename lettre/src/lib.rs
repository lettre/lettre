//! Lettre is a mailer written in Rust. It provides a simple email builder and several transports.
//!
//! ## Overview
//!
//! This mailer contains the available transports for your emails. To be sendable, the
//! emails have to implement `SendableEmail`.
//!
//! The following sections describe the available transport methods to handle emails.
//!
//! * The `SmtpTransport` uses the SMTP protocol to send the message over the network. It is
//!   the prefered way of sending emails.
//! * The `FileTransport` creates a file containing the email content to be sent. It can be used
//!   for debugging or if you want to keep all sent emails.
//! * The `StubTransport` is useful for debugging, and only prints the content of the email in the
//!   logs.
//!
//! ### SMTP transport
//!
//! This SMTP follows [RFC
//! 5321](https://tools.ietf.org/html/rfc5321), but is still
//! a work in progress. It is designed to efficiently send emails from an
//! application to a
//! relay email server, as it relies as much as possible on the relay server
//! for sanity and RFC
//! compliance checks.
//!
//! It implements the following extensions:
//!
//! * 8BITMIME ([RFC 6152](https://tools.ietf.org/html/rfc6152))
//! * AUTH ([RFC 4954](http://tools.ietf.org/html/rfc4954)) with PLAIN and
//! CRAM-MD5 mechanisms
//! * STARTTLS ([RFC 2487](http://tools.ietf.org/html/rfc2487))
//! * SMTPUTF8 ([RFC 6531](http://tools.ietf.org/html/rfc6531))
//!
//! #### Simple example
//!
//! This is the most basic example of usage:
//!
//! ```rust,no_run
//! use lettre::{SimpleSendableEmail, EmailTransport};
//! use lettre::smtp::SmtpTransportBuilder;
//! use lettre::smtp::SecurityLevel;
//!
//! let email = SimpleSendableEmail::new(
//!                 "user@localhost",
//!                 vec!["root@localhost"],
//!                 "message_id",
//!                 "Hello world"
//!             );
//!
//! // Open a local connection on port 25
//! let mut mailer =
//! SmtpTransportBuilder::localhost().unwrap().security_level(SecurityLevel::Opportunistic).build();
//! // Send the email
//! let result = mailer.send(email);
//!
//! assert!(result.is_ok());
//! ```
//!
//! #### Complete example
//!
//! ```rust,no_run
//! use lettre::smtp::{SecurityLevel, SmtpTransport,
//! SmtpTransportBuilder};
//! use lettre::smtp::authentication::Mechanism;
//! use lettre::smtp::SUBMISSION_PORT;
//! use lettre::{SimpleSendableEmail, EmailTransport};
//!
//! let email = SimpleSendableEmail::new(
//!                 "user@localhost",
//!                 vec!["root@localhost"],
//!                 "message_id",
//!                 "Hello world"
//!             );
//!
//! // Connect to a remote server on a custom port
//! let mut mailer = SmtpTransportBuilder::new(("server.tld",
//! SUBMISSION_PORT)).unwrap()
//!     // Set the name sent during EHLO/HELO, default is `localhost`
//!     .hello_name("my.hostname.tld")
//!     // Add credentials for authentication
//!     .credentials("username", "password")
//!     // Specify a TLS security level. You can also specify an SslContext with
//!     // .ssl_context(SslContext::Ssl23)
//!     .security_level(SecurityLevel::AlwaysEncrypt)
//!     // Enable SMTPUTF8 if the server supports it
//!     .smtp_utf8(true)
//!     // Configure expected authentication mechanism
//!     .authentication_mechanism(Mechanism::CramMd5)
//!     // Enable connection reuse
//!     .connection_reuse(true).build();
//!
//! let result_1 = mailer.send(email.clone());
//! assert!(result_1.is_ok());
//!
//! // The second email will use the same connection
//! let result_2 = mailer.send(email);
//! assert!(result_2.is_ok());
//!
//! // Explicitly close the SMTP transaction as we enabled connection reuse
//! mailer.close();
//! ```
//!
//! #### Lower level
//!
//! You can also send commands, here is a simple email transaction without
//! error handling:
//!
//! ```rust
//! use lettre::smtp::SMTP_PORT;
//! use lettre::smtp::client::Client;
//! use lettre::smtp::client::net::NetworkStream;
//!
//! let mut email_client: Client<NetworkStream> = Client::new();
//! let _ = email_client.connect(&("localhost", SMTP_PORT), None);
//! let _ = email_client.ehlo("my_hostname");
//! let _ = email_client.mail("user@example.com", None);
//! let _ = email_client.rcpt("user@example.org");
//! let _ = email_client.data();
//! let _ = email_client.message("Test email");
//! let _ = email_client.quit();
//! ```
//!
//! ### Sendmail transport
//!
//! The sendmail transport sends the email using the local sendmail command.
//!
//! ```rust
//! use lettre::sendmail::SendmailTransport;
//! use lettre::{SimpleSendableEmail, EmailTransport};
//!
//! let email = SimpleSendableEmail::new(
//!                 "user@localhost",
//!                 vec!["root@localhost"],
//!                 "message_id",
//!                 "Hello world"
//!             );
//!
//! let mut sender = SendmailTransport::new();
//! let result = sender.send(email);
//! assert!(result.is_ok());
//! ```
//!
//! ### Stub transport
//!
//! The stub transport only logs message envelope and drops the content. It can be useful for
//! testing purposes.
//!
//! ```rust
//! use lettre::stub::StubEmailTransport;
//! use lettre::{SimpleSendableEmail, EmailTransport};
//!
//! let email = SimpleSendableEmail::new(
//!                 "user@localhost",
//!                 vec!["root@localhost"],
//!                 "message_id",
//!                 "Hello world"
//!             );
//!
//! let mut sender = StubEmailTransport;
//! let result = sender.send(email);
//! assert!(result.is_ok());
//! ```
//!
//! Will log the line:
//!
//! ```text
//! b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
//! ```
//!
//! ### File transport
//!
//! The file transport writes the emails to the given directory. The name of the file will be
//! `message_id.txt`.
//! It can be useful for testing purposes, or if you want to keep track of sent messages.
//!
//! ```rust
//! use std::env::temp_dir;
//!
//! use lettre::file::FileEmailTransport;
//! use lettre::{SimpleSendableEmail, EmailTransport};
//!
//! // Write to the local temp directory
//! let mut sender = FileEmailTransport::new(temp_dir());
//! let email = SimpleSendableEmail::new(
//!                 "user@localhost",
//!                 vec!["root@localhost"],
//!                 "message_id",
//!                 "Hello world"
//!             );
//!
//! let result = sender.send(email);
//! assert!(result.is_ok());
//! ```
//! Example result in `/tmp/b7c211bc-9811-45ce-8cd9-68eab575d695.txt`:
//!
//! ```text
//! b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
//! To: <root@localhost>
//! From: <user@localhost>
//! Subject: Hello
//! Date: Sat, 31 Oct 2015 13:42:19 +0100
//! Message-ID: <b7c211bc-9811-45ce-8cd9-68eab575d695.lettre@localhost>
//!
//! Hello World!
//! ```

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
