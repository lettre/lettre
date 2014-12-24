// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! TODO

use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use std::str::from_utf8;
use std::vec::Vec;
use std::error::FromError;

use error::SmtpResult;
use response::Response;
use tools::{escape_crlf, escape_dot};

static BUFFER_SIZE: uint = 1024;

/// TODO
pub trait ClientStream {
    /// TODO
    fn send_and_get_response(&mut self, string: &str, end: &str) -> SmtpResult;
    /// TODO
    fn get_reply(&mut self) -> SmtpResult;
    /// TODO
    fn read_into_string(&mut self) -> IoResult<String>;
}

impl ClientStream for TcpStream {
    /// Sends a string to the server and gets the response
    fn send_and_get_response(&mut self, string: &str, end: &str) -> SmtpResult {
        try!(self.write_str(format!("{}{}", escape_dot(string), end).as_slice()));

        debug!("Wrote: {}", escape_crlf(escape_dot(string).as_slice()));

        self.get_reply()
    }

    /// Reads on the stream into a string
    fn read_into_string(&mut self) -> IoResult<String> {
        let mut more = true;
        let mut result = String::new();
        // TODO: Set appropriate timeouts
        self.set_timeout(Some(1000));

        while more {
            let mut buf: Vec<u8> = Vec::with_capacity(BUFFER_SIZE);
            let response = match self.push(BUFFER_SIZE, &mut buf) {
                Ok(bytes_read) => {
                    more = bytes_read == BUFFER_SIZE;
                    if bytes_read > 0 {
                        from_utf8(buf.slice_to(bytes_read)).unwrap()
                    } else {
                        ""
                    }
                },
                // TODO: Manage error kinds
                Err(..) => {more = false; ""},
            };
            result.push_str(response);
        }
        debug!("Read: {}", escape_crlf(result.as_slice()));
        return Ok(result);
    }

    /// Gets the SMTP response
    fn get_reply(&mut self) -> SmtpResult {
        let response = try!(self.read_into_string());

        match response.as_slice().parse::<Response>() {
            Some(response) => Ok(response),
            None => Err(FromError::from_error("Could not parse response"))
        }
    }
}
