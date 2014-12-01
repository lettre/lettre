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
//! ### Basic usage
//!
//! If you just want to send an email:
//!
//! ```rust,no_run
//! use std::io::net::tcp::TcpStream;
//! use smtp::client::Client;
//! use smtp::common::SMTP_PORT;
//!
//! let mut email_client: Client<TcpStream> =
//!     Client::new(
//!         ("localhost", SMTP_PORT), // server socket
//!         Some("myhost"),           // my hostname (default is localhost)
//!     );
//! let result = email_client.send_mail::<TcpStream>(
//!     "user@example.com",   // sender (reverse-path)
//!     &["user@example.org"], // recipient list
//!     "Test email",         // email content
//! );
//! ```
//!
//! ### Lower level
//!
//! You can also send commands, here is a simple email transaction without error handling:
//!
//! ```rust,no_run
//! use std::io::net::tcp::TcpStream;
//! use smtp::client::Client;
//! use smtp::common::SMTP_PORT;
//!
//! let mut email_client: Client<TcpStream> =
//!     Client::new(
//!         ("localhost", SMTP_PORT), // server socket
//!         Some("myhost"),           // my hostname (default is localhost)
//!     );
//! let _ = email_client.connect();
//! let _ = email_client.ehlo::<TcpStream>();
//! let _ = email_client.mail::<TcpStream>("user@example.com");
//! let _ = email_client.rcpt::<TcpStream>("user@example.org");
//! let _ = email_client.data::<TcpStream>();
//! let _ = email_client.message::<TcpStream>("Test email");
//! let _ = email_client.quit::<TcpStream>();
//! ```

#![crate_type = "lib"]

#![desc = "Rust SMTP library"]
#![comment = "Simple SMTP client and library"]
#![license = "MIT/ASL2"]
#![doc(html_root_url = "http://amousset.github.io/rust-smtp/smtp/")]
#![experimental]

#![feature(phase, macro_rules)]
#![deny(missing_docs, warnings)]

#![feature(phase)] #[phase(plugin, link)] extern crate log;

pub mod client;
pub mod command;
pub mod extension;
pub mod response;
pub mod transaction;
pub mod common;
pub mod error;
pub mod tools;
