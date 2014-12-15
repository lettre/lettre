// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Rust SMTP library
//!
//! The client should tend to follow [RFC 5321](https://tools.ietf.org/html/rfc5321), but is still
//! a work in progress.
//!
//! It may eventually implement the following extensions :
//!
//! * 8BITMIME ([RFC 6152](https://tools.ietf.org/html/rfc6152))
//! * SMTPUTF8 ([RFC 6531](http://tools.ietf.org/html/rfc6531))
//! * STARTTLS ([RFC 2487](http://tools.ietf.org/html/rfc2487))
//! * AUTH ([RFC 4954](http://tools.ietf.org/html/rfc4954))
//!
//! ## Usage
//!
//! ### Simple example
//!
//! This is the most basic example of usage:
//!
//! ```rust,no_run
//! #![feature(default_type_params)]
//! use smtp::client::Client;
//! use smtp::mailer::Email;
//!
//! // Create an email
//! let mut email = Email::new();
//! // Addresses can be specified by the couple (email, alias)
//! email.to(("user@example.org", "Firstname Lastname"));
//! // ... or by an address only
//! email.from("user@example.com");
//! email.subject("Hello world");
//! email.body("Hi, Hello world.");
//! email.date_now();
//!
//! // Open a local connection on port 25
//! let mut client = Client::localhost();
//! // Send the email
//! let result = client.send(email);
//!
//! assert!(result.is_ok());
//! ```
//!
//! You can send multiple emails using the same connection by using `send` several times on the
//! same client. If the connection was closed, it will be re-opened.
//!
//! ### Complete example
//!
//! ```rust,no_run
//! #![feature(default_type_params)]
//! use smtp::client::Client;
//! use smtp::mailer::Email;
//!
//! let mut email = Email::new();
//! email.to(("user@example.org", "Alias name"));
//! email.cc(("user@example.net", "Alias name"));
//! email.from("no-reply@example.com");
//! email.from("no-reply@example.eu");
//! email.sender("no-reply@example.com");
//! email.subject("Hello world");
//! email.body("Hi, Hello world.");
//! email.reply_to("contact@example.com");
//! email.add_header(("X-Custom-Header", "my header"));
//! email.date_now();
//!
//! let mut client = Client::new(
//!     ("server.tld", 10025),   // remote server and custom port
//!     Some("my.hostname.tld"), // my hostname
//! );
//! let result = client.send(email);
//! assert!(result.is_ok());
//! ```
//!
//! ### Using the client directly
//!
//! If you just want to send an email without using `Email` to provide headers:
//!
//! ```rust,no_run
//! #![feature(default_type_params)]
//! use smtp::client::Client;
//! use smtp::sendable_email::SimpleSendableEmail;
//!
//! // Create a minimal email
//! let email = SimpleSendableEmail::new(
//!     "test@example.com",
//!     "test@example.org",
//!     "Hello world !"
//! );
//!
//! let mut client = Client::new(
//!     "localhost",             // server socket
//!     Some("my.hostname.tld"), // my hostname (default is localhost)
//! );
//! let result = client.send(email);
//! assert!(result.is_ok());
//! ```
//!
//! ### Lower level
//!
//! You can also send commands, here is a simple email transaction without error handling:
//!
//! ```rust,no_run
//! #![feature(default_type_params)]
//! use smtp::client::Client;
//! use smtp::common::SMTP_PORT;
//!
//! let mut email_client = Client::new(
//!     ("localhost", SMTP_PORT), // server socket
//!     Some("my.hostname.tld"),  // my hostname (default is localhost)
//! );
//! let _ = email_client.connect();
//! let _ = email_client.ehlo();
//! let _ = email_client.mail("user@example.com");
//! let _ = email_client.rcpt("user@example.org");
//! let _ = email_client.data();
//! let _ = email_client.message("Test email");
//! let _ = email_client.quit();
//! ```

#![crate_type = "lib"]

#![doc(html_root_url = "http://amousset.github.io/rust-smtp/smtp/")]
#![experimental]

#![feature(phase, macro_rules, default_type_params)]
#![deny(missing_docs, warnings)]

#![feature(phase)] #[phase(plugin, link)] extern crate log;

extern crate time;

pub mod client;
pub mod command;
pub mod extension;
pub mod response;
pub mod transaction;
pub mod common;
pub mod error;
pub mod tools;
pub mod sendable_email;
pub mod mailer;
