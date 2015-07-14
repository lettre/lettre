// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP client

use std::string::String;
use std::net::{SocketAddr, ToSocketAddrs};
use std::io::{BufRead, Read, Write};

use bufstream::BufStream;

use response::ResponseParser;
use error::{Error, SmtpResult};
use client::net::{Connector, SmtpStream};
use client::authentication::{plain, cram_md5};
use {CRLF, MESSAGE_ENDING};

pub mod net;
mod authentication;

/// Returns the string after adding a dot at the beginning of each line starting with a dot
///
/// Reference : https://tools.ietf.org/html/rfc5321#page-62 (4.5.2. Transparency)
#[inline]
fn escape_dot(string: &str) -> String {
    if string.starts_with(".") {
        format!(".{}", string)
    } else {
        string.to_string()
    }.replace("\r.", "\r..")
     .replace("\n.", "\n..")
}

/// Returns the string replacing all the CRLF with "\<CRLF\>"
#[inline]
fn escape_crlf(string: &str) -> String {
    string.replace(CRLF, "<CR><LF>")
}

/// Returns the string removing all the CRLF
#[inline]
fn remove_crlf(string: &str) -> String {
    string.replace(CRLF, "")
}

/// Structure that implements the SMTP client
pub struct Client<S: Write + Read = SmtpStream> {
    /// TCP stream between client and server
    /// Value is None before connection
    stream: Option<BufStream<S>>,
    /// Socket we are connecting to
    server_addr: SocketAddr,

}

macro_rules! return_err (
    ($err: expr, $client: ident) => ({
        return Err(From::from($err))
    })
);

impl<S: Write + Read = SmtpStream> Client<S> {
    /// Creates a new SMTP client
    ///
    /// It does not connects to the server, but only creates the `Client`
    pub fn new<A: ToSocketAddrs>(addr: A) -> Client<S> {
        Client{
            stream: None,
            server_addr: addr.to_socket_addrs().ok().expect("could not parse server address").next().unwrap(),
        }
    }
}

impl<S: Connector + Write + Read = SmtpStream> Client<S> {
    /// Closes the SMTP transaction if possible
    pub fn close(&mut self) {
        let _ = self.quit();
        self.stream = None;
    }

    /// Connects to the configured server
    pub fn connect(&mut self) -> SmtpResult {
        // Connect should not be called when the client is already connected
        if self.stream.is_some() {
            return_err!("The connection is already established", self);
        }

        // Try to connect
        self.stream = Some(BufStream::new(try!(Connector::connect(&self.server_addr))));

        self.get_reply()
    }

    /// Checks if the server is connected using the NOOP SMTP command
    pub fn is_connected(&mut self) -> bool {
        self.noop().is_ok()
    }

    /// Sends an SMTP command
    pub fn command(&mut self, command: &str) -> SmtpResult {
        self.send_server(command, CRLF)
    }

    /// Send a HELO command and fills `server_info`
    pub fn helo(&mut self, hostname: &str) -> SmtpResult {
        self.command(&format!("HELO {}", hostname))
    }

    /// Sends a EHLO command and fills `server_info`
    pub fn ehlo(&mut self, hostname: &str) -> SmtpResult {
        self.command(&format!("EHLO {}", hostname))
    }

    /// Sends a MAIL command
    pub fn mail(&mut self, address: &str, options: Option<&str>) -> SmtpResult {
        match options {
            Some(ref options) => self.command(&format!("MAIL FROM:<{}> {}", address, options)),
            None => self.command(&format!("MAIL FROM:<{}>", address)),
        }
    }

    /// Sends a RCPT command
    pub fn rcpt(&mut self, address: &str) -> SmtpResult {
        self.command(&format!("RCPT TO:<{}>", address))
    }

    /// Sends a DATA command
    pub fn data(&mut self) -> SmtpResult {
        self.command("DATA")
    }

    /// Sends a QUIT command
    pub fn quit(&mut self) -> SmtpResult {
        self.command("QUIT")
    }

    /// Sends a NOOP command
    pub fn noop(&mut self) -> SmtpResult {
        self.command("NOOP")
    }

    /// Sends a HELP command
    pub fn help(&mut self, argument: Option<&str>) -> SmtpResult {
        match argument {
            Some(ref argument) => self.command(&format!("HELP {}", argument)),
            None => self.command("HELP"),
        }
    }

    /// Sends a VRFY command
    pub fn vrfy(&mut self, address: &str) -> SmtpResult {
        self.command(&format!("VRFY {}", address))
    }

    /// Sends a EXPN command
    pub fn expn(&mut self, address: &str) -> SmtpResult {
        self.command(&format!("EXPN {}", address))
    }

    /// Sends a RSET command
    pub fn rset(&mut self) -> SmtpResult {
        self.command("RSET")
    }

    /// Sends an AUTH command with PLAIN mecanism
    pub fn auth_plain(&mut self, username: &str, password: &str) -> SmtpResult {
        self.command(&format!("AUTH PLAIN {}", plain(username, password)))
    }

    /// Sends an AUTH command with CRAM-MD5 mecanism
    pub fn auth_cram_md5(&mut self, username: &str, password: &str) -> SmtpResult {
        let encoded_challenge = match try!(self.command("AUTH CRAM-MD5")).first_word() {
            Some(challenge) => challenge,
            None => return Err(Error::ResponseParsingError("Could not read CRAM challenge")),
        };

        let cram_response = try!(cram_md5(username, password, &encoded_challenge));

        self.command(&format!("AUTH CRAM-MD5 {}", cram_response))
    }

    /// Sends the message content
    pub fn message(&mut self, message_content: &str) -> SmtpResult {
        self.send_server(&escape_dot(message_content), MESSAGE_ENDING)
    }

    /// Sends a string to the server and gets the response
    fn send_server(&mut self, string: &str, end: &str) -> SmtpResult {
        if self.stream.is_none() {
            return Err(From::from("Connection closed"));
        }

        try!(write!(self.stream.as_mut().unwrap(), "{}{}", string, end));
        try!(self.stream.as_mut().unwrap().flush());

        debug!("Wrote: {}", escape_crlf(string));

        self.get_reply()
    }

    /// Gets the SMTP response
    fn get_reply(&mut self) -> SmtpResult {

        let mut parser = ResponseParser::new();

        let mut line = String::new();
        try!(self.stream.as_mut().unwrap().read_line(&mut line));

        while try!(parser.read_line(remove_crlf(line.as_ref()).as_ref())) {
            line.clear();
            try!(self.stream.as_mut().unwrap().read_line(&mut line));
        }

        let response = try!(parser.response());

        match response.is_positive() {
            true => Ok(response),
            false => Err(From::from(response)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{escape_dot, remove_crlf, escape_crlf};

    #[test]
    fn test_escape_dot() {
        assert_eq!(escape_dot(".test"), "..test");
        assert_eq!(escape_dot("\r.\n.\r\n"), "\r..\n..\r\n");
        assert_eq!(escape_dot("test\r\n.test\r\n"), "test\r\n..test\r\n");
        assert_eq!(escape_dot("test\r\n.\r\ntest"), "test\r\n..\r\ntest");
    }

    #[test]
    fn test_remove_crlf() {
        assert_eq!(remove_crlf("\r\n"), "");
        assert_eq!(remove_crlf("EHLO my_name\r\n"), "EHLO my_name");
        assert_eq!(
            remove_crlf("EHLO my_name\r\nSIZE 42\r\n"),
            "EHLO my_nameSIZE 42"
        );
    }

    #[test]
    fn test_escape_crlf() {
        assert_eq!(escape_crlf("\r\n"), "<CR><LF>");
        assert_eq!(escape_crlf("EHLO my_name\r\n"), "EHLO my_name<CR><LF>");
        assert_eq!(
            escape_crlf("EHLO my_name\r\nSIZE 42\r\n"),
            "EHLO my_name<CR><LF>SIZE 42<CR><LF>"
        );
    }
}
