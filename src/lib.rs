// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Rust SMTP client
//!
//! This client should tend to follow [RFC 5321](https://tools.ietf.org/html/rfc5321), but is still
//! a work in progress. It is designed to efficiently send emails from a rust application to a
//! relay email server.
//!
//! It implements the following extensions:
//!
//! * 8BITMIME ([RFC 6152](https://tools.ietf.org/html/rfc6152))
//! * AUTH ([RFC 4954](http://tools.ietf.org/html/rfc4954))
//!
//! It will eventually implement the following extensions:
//!
//! * STARTTLS ([RFC 2487](http://tools.ietf.org/html/rfc2487))
//! * SMTPUTF8 ([RFC 6531](http://tools.ietf.org/html/rfc6531))
//!
//! ## Architecture
//!
//! This client is divided into three parts:
//!
//! * client: a low level SMTP client providing all SMTP commands
//! * sender: a high level SMTP client providing an easy method to send emails
//! * email: generates the email to be sent with the sender
//!
//! ## Usage
//!
//! ### Simple example
//!
//! This is the most basic example of usage:
//!
//! ```rust,no_run
//! use smtp::sender::{Sender, SenderBuilder};
//! use smtp::email::EmailBuilder;
//!
//! // Create an email
//! let email = EmailBuilder::new()
//!     // Addresses can be specified by the couple (email, alias)
//!     .to(("user@example.org", "Firstname Lastname"))
//!     // ... or by an address only
//!     .from("user@example.com")
//!     .subject("Hi, Hello world")
//!     .body("Hello world.")
//!     .build();
//!
//! // Open a local connection on port 25
//! let mut sender = SenderBuilder::localhost().build();
//! // Send the email
//! let result = sender.send(email);
//!
//! assert!(result.is_ok());
//! ```
//!
//! ### Complete example
//!
//! ```rust,no_run
//! use smtp::sender::{Sender, SenderBuilder};
//! use smtp::email::EmailBuilder;
//!
//! let mut builder = EmailBuilder::new();
//! builder = builder.to(("user@example.org", "Alias name"));
//! builder = builder.cc(("user@example.net", "Alias name"));
//! builder = builder.from("no-reply@example.com");
//! builder = builder.from("no-reply@example.eu");
//! builder = builder.sender("no-reply@example.com");
//! builder = builder.subject("Hello world");
//! builder = builder.body("Hi, Hello world.");
//! builder = builder.reply_to("contact@example.com");
//! builder = builder.add_header(("X-Custom-Header", "my header"));
//!
//! let email = builder.build();
//!
//! // Connect to a remote server on a custom port
//! let mut sender = SenderBuilder::new(("server.tld", 10025))
//!     // Set the name sent during EHLO/HELO, default is `localhost`
//!     .hello_name("my.hostname.tld")
//!     // Add credentials for authentication
//!     .credentials("username", "password")
//!     // Enable connection reuse
//!     .enable_connection_reuse(true).build();
//!
//! let result_1 = sender.send(email.clone());
//! assert!(result_1.is_ok());
//!
//! // The second email will use the same connection
//! let result_2 = sender.send(email);
//! assert!(result_2.is_ok());
//!
//! // Explicitely close the SMTP transaction as we enabled connection reuse
//! sender.close();
//! ```
//!
//! ### Using the client directly
//!
//! If you just want to send an email without using `Email` to provide headers:
//!
//! ```rust,no_run
//! use smtp::sender::{Sender, SenderBuilder};
//! use smtp::sendable_email::SimpleSendableEmail;
//!
//! // Create a minimal email
//! let email = SimpleSendableEmail::new(
//!     "test@example.com",
//!     "test@example.org",
//!     "Hello world !"
//! );
//!
//! let mut sender = SenderBuilder::localhost().build();
//! let result = sender.send(email);
//! assert!(result.is_ok());
//! ```
//!
//! ### Lower level
//!
//! You can also send commands, here is a simple email transaction without error handling:
//!
//! ```rust,no_run
//! use smtp::client::Client;
//! use smtp::client::net::SmtpStream;
//! use smtp::SMTP_PORT;
//! use std::net::TcpStream;
//!
//! let mut email_client: Client<SmtpStream> = Client::new(("localhost", SMTP_PORT));
//! let _ = email_client.connect();
//! let _ = email_client.ehlo("my_hostname");
//! let _ = email_client.mail("user@example.com", None);
//! let _ = email_client.rcpt("user@example.org");
//! let _ = email_client.data();
//! let _ = email_client.message("Test email");
//! let _ = email_client.quit();
//! ```

#![deny(missing_docs)]

#[macro_use] extern crate log;
extern crate rustc_serialize as serialize;
extern crate crypto;
extern crate time;
extern crate uuid;
extern crate email as email_format;
extern crate bufstream;

mod extension;
pub mod client;
pub mod sender;
pub mod response;
pub mod error;
pub mod sendable_email;
pub mod email;

// Registrated port numbers:
// https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml

/// Default smtp port
pub static SMTP_PORT: u16 = 25;

/// Default smtps port
pub static SMTPS_PORT: u16 = 465;

/// Default submission port
pub static SUBMISSION_PORT: u16 = 587;

// Useful strings and characters

/// The word separator for SMTP transactions
pub static SP: &'static str = " ";

/// The line ending for SMTP transactions (carriage return + line feed)
pub static CRLF: &'static str = "\r\n";

/// Colon
pub static COLON: &'static str = ":";

/// The ending of message content
pub static MESSAGE_ENDING: &'static str = "\r\n.\r\n";

/// NUL unicode character
pub static NUL: &'static str = "\0";
