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
//! The client does its best to follow RFC 5321 (https://tools.ietf.org/html/rfc5321).
//!
//! It also implements the following extensions :
//!
//! * 8BITMIME (RFC 6152 : https://tools.ietf.org/html/rfc6152)
//! * SIZE (RFC 1427 : https://tools.ietf.org/html/rfc1427)
//!
//! ## What this client is NOT made for
//!
//! Send emails to public email servers. It is not designed to smartly handle servers responses,
//! to rate-limit emails, to make retries, and all that complicated stuff needed to politely
//! talk to public servers.
//!
//! What this client does is basically try once to send the email, and say if it worked.
//! It should only be used to transfer emails to a relay server.
//!
//! ## Usage
//! 
//! ```rust
//! extern crate smtp;
//! use std::io::net::tcp::TcpStream;
//! use smtp::client::SmtpClient;
//! use std::strbuf::StrBuf;
//!
//! let mut email_client: SmtpClient<StrBuf, TcpStream> = 
//!     SmtpClient::new(StrBuf::from_str("localhost"), None, None);
//! email_client.send_mail(
//!     StrBuf::from_str("user@example.com"),
//!     vec!(StrBuf::from_str("user@example.org")),
//!     StrBuf::from_str("Test email")
//! );
//! ```

// May  4 21:48:18 mx1 smtp-gmail/smtp[12657]: Untrusted TLS connection established to gmail-smtp-in.l.google.com[173.194.66.26]:25: TLSv1.2 with cipher ECDHE-RSA-AES128-GCM-SHA256 (128/128 bits)
// May  4 21:48:18 mx1 smtp-gmail/smtp[12657]: 69800746A7B: to=<kelvin.moutet@gmail.com>, orig_to=<kelvin@minet.net>, relay=gmail-smtp-in.l.google.com[173.194.66.26]:25, delay=2.6, delays=1.7/0.03/0.19/0.61, dsn=2.0.0, status=sent (250 2.0.0 OK 1399232898 fy10si2284441wib.22 - gsmtp)

#![crate_id = "smtp#0.1-pre"]

#![desc = "Rust SMTP client"]
#![comment = "Simple SMTP client, without AUTH or SSL/TLS for now"]
#![license = "MIT/ASL2"]
#![crate_type = "lib"]


#![feature(macro_rules)]
#![deny(non_camel_case_types)]
#![deny(missing_doc)]

#![feature(phase)]
#[phase(syntax, link)] extern crate log;

pub mod commands;
pub mod common;
pub mod client;
