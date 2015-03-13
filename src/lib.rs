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
//! It implements the following extensions :
//!
//! * 8BITMIME ([RFC 6152](https://tools.ietf.org/html/rfc6152))
//! * AUTH ([RFC 4954](http://tools.ietf.org/html/rfc4954))
//!
//! It will eventually implement the following extensions :
//!
//! * STARTTLS ([RFC 2487](http://tools.ietf.org/html/rfc2487))
//! * SMTPUTF8 ([RFC 6531](http://tools.ietf.org/html/rfc6531))
//!
//! ## Usage
//!
//! ### Simple example
//!
//! This is the most basic example of usage:
//!
//! ```rust,no_run
//! use smtp::client::{Client, ClientBuilder};
//! use smtp::mailer::EmailBuilder;
//! use std::net::TcpStream;
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
//! let mut client: Client<TcpStream> = ClientBuilder::localhost().build();
//! // Send the email
//! let result = client.send(email);
//!
//! assert!(result.is_ok());
//! ```
//!
//! ### Complete example
//!
//! ```rust,no_run
//! use smtp::client::{Client, ClientBuilder};
//! use smtp::mailer::EmailBuilder;
//! use std::net::TcpStream;
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
//! let mut client: Client<TcpStream> = ClientBuilder::new(("server.tld", 10025))
//!     // Set the name sent during EHLO/HELO, default is `localhost`
//!     .hello_name("my.hostname.tld")
//!     // Add credentials for authentication
//!     .credentials("username", "password")
//!     // Enable connection reuse
//!     .enable_connection_reuse(true).build();
//! let result_1 = client.send(email.clone());
//! assert!(result_1.is_ok());
//! // The second email will use the same connection
//! let result_2 = client.send(email);
//! assert!(result_2.is_ok());
//!
//! // Explicitely close the SMTP transaction as we enabled connection reuse
//! client.close();
//! ```
//!
//! ### Using the client directly
//!
//! If you just want to send an email without using `Email` to provide headers:
//!
//! ```rust,no_run
//! use smtp::client::{Client, ClientBuilder};
//! use smtp::sendable_email::SimpleSendableEmail;
//! use std::net::TcpStream;
//!
//! // Create a minimal email
//! let email = SimpleSendableEmail::new(
//!     "test@example.com",
//!     "test@example.org",
//!     "Hello world !"
//! );
//!
//! let mut client: Client<TcpStream> = ClientBuilder::localhost().build();
//! let result = client.send(email);
//! assert!(result.is_ok());
//! ```
//!
//! ### Lower level
//!
//! You can also send commands, here is a simple email transaction without error handling:
//!
//! ```rust,no_run
//! use smtp::client::{Client, ClientBuilder};
//! use smtp::SMTP_PORT;
//! use std::net::TcpStream;
//!
//! let mut email_client: Client<TcpStream> = ClientBuilder::new(("localhost", SMTP_PORT)).build();
//! let _ = email_client.connect();
//! let _ = email_client.ehlo();
//! let _ = email_client.mail("user@example.com");
//! let _ = email_client.rcpt("user@example.org");
//! let _ = email_client.data();
//! let _ = email_client.message("Test email");
//! let _ = email_client.quit();
//! ```


#![feature(plugin, core, io, collections, net, str_words)]
#![deny(missing_docs)]

#[macro_use] extern crate log;
extern crate "rustc-serialize" as serialize;
extern crate crypto;
extern crate time;
extern crate uuid;

mod tools;
mod extension;
pub mod client;
pub mod response;
pub mod error;
pub mod sendable_email;
pub mod mailer;

// Registrated port numbers:
// https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml

/// Default smtp port
pub static SMTP_PORT: u16 = 25;

/// Default smtps port
pub static SMTPS_PORT: u16 = 465;

/// Default submission port
pub static SUBMISSION_PORT: u16 = 587;
