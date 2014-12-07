// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simple email (very incomplete)

use std::fmt::{Show, Formatter, Result};

use time::{now, Tm};

use email::header::{ToHeader, Header};
use email::address::ToAddress;
use common::CRLF;

pub mod header;
pub mod address;

/// Simple email representation
pub struct Email {
    /// Array of headers
    headers: Vec<Header>,
    /// Message body
    body: String,
    /// The enveloppe recipients addresses
    to: Vec<String>,
    /// The enveloppe sender address
    from: Option<String>,
}

impl Show for Email {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut formatted_headers = String::new();
        for header in self.headers.iter() {
            formatted_headers.push_str(header.to_string().as_slice());
            formatted_headers.push_str(CRLF);
        }
        f.write(format!("{}{}{}", formatted_headers, CRLF, self.body).as_bytes())
    }
}

impl Email {
    /// Creates a new empty email
    pub fn new() -> Email {
        Email{headers: vec!(), body: "".to_string(), to: vec!(), from: None}
    }

    /// Clear the email content
    pub fn clear(&mut self) {
        self.headers.clear();
        self.body = "".to_string();
        self.to.clear();
        self.from = None;
    }

    /// Return the to addresses, and fails if it is not set
    pub fn get_to<'a>(&'a self) -> Vec<String> {
        if self.to.is_empty() {
            panic!("The To field is empty")
        }
        self.to.clone()
    }

    /// Return the from address, and fails if it is not set
    pub fn get_from(&self) -> String {
        match self.from {
            Some(ref from_address) => from_address.clone(),
            None => panic!("The From field is empty"),
        }
    }

    /// Sets the email body
    pub fn body(&mut self, body: &str) {
        self.body = body.to_string();
    }

    /// Add a generic header
    pub fn add_header<A: ToHeader>(&mut self, header: A) {
        self.headers.push(header.to_header());
    }

    /// Adds a From header and store the sender address
    pub fn from<A: ToAddress>(&mut self, address: A) {
        self.from = Some(address.to_address().get_address());
        self.headers.push(
            Header::new("From", address.to_address().to_string().as_slice())
        );
    }

    /// Adds a To header and store the recipient address
    pub fn to<A: ToAddress>(&mut self, address: A) {
        self.to.push(address.to_address().get_address());
        self.headers.push(
            Header::new("To", address.to_address().to_string().as_slice())
        );
    }

    /// Adds a Cc header and store the recipient address
    pub fn cc<A: ToAddress>(&mut self, address: A) {
        self.to.push(address.to_address().get_address());
        self.headers.push(
            Header::new("Cc", address.to_address().to_string().as_slice())
        );
    }

    /// Adds a Reply-To header
    pub fn reply_to<A: ToAddress>(&mut self, address: A) {
        self.headers.push(
            Header::new("Return-Path", address.to_address().to_string().as_slice())
        );
    }

    /// Adds a Subject header
    pub fn subject(&mut self, subject: &str) {
        self.headers.push(
            Header::new("Subject", subject)
        );
    }

    /// Adds a Date header with the current time
    pub fn date(&mut self) {
        self.headers.push(
            Header::new("Date", Tm::rfc822(&now()).to_string().as_slice())
        );
    }
}
