// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simple email

use std::fmt::{Show, Formatter, Result};

use time::{now, Tm};

use email::header::Header;
use email::address::ToAddress;
use common::CRLF;
//use client::Client;
//use error::SmtpResult;

pub mod header;
pub mod address;

/// TODO
pub struct Email {
    /// Array of headers
    headers: Vec<Header>,
    /// Message body
    body: Option<String>,
    /// TODO cc bcc to
    to: Vec<String>,
    /// TODO
    from: Option<String>,
}

impl Show for Email {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write(format!("{}{}{}", self.headers, CRLF, self.body).as_bytes())
    }
}

impl Email {
    /// TODO
    // pub fn send(&self, client: Client) -> SmtpResult {
    //     let test: Vec<&str> = self.to.iter().map(|s| s.as_slice()).collect();
    //     //let to_vec: &[&str] = self.to.iter().map(|s| s.as_slice()).collect().as_slice();
    //     client.send_mail(
    //         self.from.unwrap().as_slice(),
    //         test.as_slice(),
    //         self.to_string().as_slice(),
    //     )
    // }

    /// TODO
    pub fn new() -> Email {
        Email{headers: vec!(), body: None, to: vec!(), from: None}
    }

    // TODO : standard headers method

    /// TODO
    pub fn body(&mut self, body: &str) {
        self.body = Some(body.to_string());
    }

    /// TODO
    pub fn add_header(&mut self, header: Header) {
        self.headers.push(header);
    }

    /// TODO
    pub fn from<A: ToAddress>(&mut self, address: A) {
        self.from = Some(address.to_address().get_address());
        self.headers.push(
            Header::new("From", address.to_address().to_string().as_slice())
        );
    }

    /// TODO
    pub fn to<A: ToAddress>(&mut self, address: A) {
        self.to.push(address.to_address().get_address());
        self.headers.push(
            Header::new("To", address.to_address().to_string().as_slice())
        );
    }

    /// TODO
    pub fn reply_to<A: ToAddress>(&mut self, address: A) {
        self.headers.push(
            Header::new("Return-Path", address.to_address().to_string().as_slice())
        );
    }

    /// TODO
    pub fn subject(&mut self, subject: &str) {
        self.headers.push(
            Header::new("Subject", subject)
        );
    }

    /// TODO
    pub fn date(&mut self) {
        self.headers.push(
            Header::new("Date", Tm::rfc822(&now()).to_string().as_slice())
        );
    }
}
