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
use std::error::FromError;
use std::net::TcpStream;
use std::net::{SocketAddr, ToSocketAddrs};
use std::io::{BufRead, BufStream, Read, Write};

use uuid::Uuid;
use serialize::base64::{self, ToBase64, FromBase64};
use serialize::hex::ToHex;
use crypto::hmac::Hmac;
use crypto::md5::Md5;
use crypto::mac::Mac;

use SMTP_PORT;
use tools::{NUL, CRLF, MESSAGE_ENDING};
use tools::{escape_dot, escape_crlf};
use response::{Response, Severity, Category};
use extension::Extension;
use error::{SmtpResult, SmtpError};
use sendable_email::SendableEmail;
use client::connecter::Connecter;
use client::server_info::ServerInfo;

mod server_info;
mod connecter;

/// Contains client configuration
pub struct ClientBuilder {
    /// Maximum connection reuse
    ///
    /// Zero means no limitation
    connection_reuse_count_limit: u16,
    /// Enable connection reuse
    enable_connection_reuse: bool,
    /// Name sent during HELO or EHLO
    hello_name: String,
    /// Credentials
    credentials: Option<(String, String)>,
    /// Socket we are connecting to
    server_addr: SocketAddr,
}

/// Builder for the SMTP Client
impl ClientBuilder {
    /// Creates a new local SMTP client
    pub fn new<A: ToSocketAddrs>(addr: A) -> ClientBuilder {
        ClientBuilder {
            server_addr: addr.to_socket_addrs().ok().expect("could not parse server address").next().unwrap(),
            credentials: None,
            connection_reuse_count_limit: 100,
            enable_connection_reuse: false,
            hello_name: "localhost".to_string(),
        }
    }

    /// Creates a new local SMTP client to port 25
    pub fn localhost() -> ClientBuilder {
        ClientBuilder::new(("localhost", SMTP_PORT))
    }

    /// Set the name used during HELO or EHLO
    pub fn hello_name(mut self, name: &str) -> ClientBuilder {
        self.hello_name = name.to_string();
        self
    }

    /// Enable connection reuse
    pub fn enable_connection_reuse(mut self, enable: bool) -> ClientBuilder {
        self.enable_connection_reuse = enable;
        self
    }

    /// Set the maximum number of emails sent using one connection
    pub fn connection_reuse_count_limit(mut self, limit: u16) -> ClientBuilder {
        self.connection_reuse_count_limit = limit;
        self
    }

    /// Set the client credentials
    pub fn credentials(mut self, username: &str, password: &str) -> ClientBuilder {
        self.credentials = Some((username.to_string(), password.to_string()));
        self
    }

    /// Build the SMTP client
    ///
    /// It does not connects to the server, but only creates the `Client`
    pub fn build<S: Connecter + Read + Write>(self) -> Client<S> {
        Client::new(self)
    }
}

/// Represents the state of a client
#[derive(Debug)]
struct State {
    /// Panic state
    pub panic: bool,
    /// Connection reuse counter
    pub connection_reuse_count: u16,
}

/// Structure that implements the SMTP client
pub struct Client<S = TcpStream> {
    /// TCP stream between client and server
    /// Value is None before connection
    stream: Option<BufStream<S>>,
    /// Information about the server
    /// Value is None before HELO/EHLO
    server_info: Option<ServerInfo>,
    /// Client variable states
    state: State,
    /// Information about the client
    client_info: ClientBuilder,
}

macro_rules! try_smtp (
    ($err: expr, $client: ident) => ({
        match $err {
            Ok(val) => val,
            Err(err) => close_and_return_err!(err, $client),
        }
    })
);

macro_rules! close_and_return_err (
    ($err: expr, $client: ident) => ({
        if !$client.state.panic {
            $client.state.panic = true;
            $client.close();
        }
        return Err(FromError::from_error($err))
    })
);

macro_rules! check_response (
    ($result: ident) => ({
        match $result {
            Ok(response) => {
                match response.is_positive() {
                    true => Ok(response),
                    false => Err(FromError::from_error(response)),
                }
            },
            Err(_) => $result,
        }
    })
);

impl<S = TcpStream> Client<S> {
    /// Creates a new SMTP client
    ///
    /// It does not connects to the server, but only creates the `Client`
    pub fn new(builder: ClientBuilder) -> Client<S> {
        Client{
            stream: None,
            server_info: None,
            client_info: builder,
            state: State {
                panic: false,
                connection_reuse_count: 0,
            },
        }
    }
}

impl<S: Connecter + Write + Read = TcpStream> Client<S> {
    /// Closes the SMTP transaction if possible
    pub fn close(&mut self) {
        let _ = self.quit();
    }

    /// Reset the client state
    fn reset(&mut self) {
        // Close the SMTP transaction if needed
        self.close();

        // Reset the client state
        self.stream = None;
        self.server_info = None;
        self.state.panic = false;
        self.state.connection_reuse_count = 0;
    }

    /// Sends an email
    pub fn send<T: SendableEmail>(&mut self, mut email: T) -> SmtpResult {
        // If there is a usable connection, test if the server answers and hello has been sent
        if self.state.connection_reuse_count > 0 {
            if !self.is_connected() {
                self.reset();
            }
        }

        // Connect to the server if needed
        if self.stream.is_none() {
            try!(self.connect());

            // Log the connection
            info!("connection established to {}", self.client_info.server_addr);

            // Extended Hello or Hello if needed
            if let Err(error) = self.ehlo() {
                match error {
                    SmtpError::PermanentError(ref response) if response.has_code(550) => {
                        try_smtp!(self.helo(), self);
                    },
                    _ => {
                        try_smtp!(Err(error), self)
                    },
                };
            }

            // Print server information
            debug!("server {}", self.server_info.as_ref().unwrap());
        }

        // TODO: Use PLAIN AUTH in encrypted connections, CRAM-MD5 otherwise
        if self.client_info.credentials.is_some() && self.state.connection_reuse_count == 0 {

            let (username, password) = self.client_info.credentials.clone().unwrap();

            if self.server_info.as_ref().unwrap().supports_feature(Extension::CramMd5Authentication) {
                let result = self.auth_cram_md5(username.as_slice(),
                                                password.as_slice());
                try_smtp!(result, self);
            } else if self.server_info.as_ref().unwrap().supports_feature(Extension::PlainAuthentication) {
                let result = self.auth_plain(username.as_slice(),
                                             password.as_slice());
                try_smtp!(result, self);
            } else {
                debug!("No supported authentication mecanisms available");
            }
        }

        let current_message = Uuid::new_v4();
        email.set_message_id(format!("<{}@{}>", current_message,
            self.client_info.hello_name.clone()));

        let from_address = email.from_address();
        let to_addresses = email.to_addresses();
        let message = email.message();

        // Mail
        try_smtp!(self.mail(from_address.as_slice()), self);

        // Log the mail command
        info!("{}: from=<{}>", current_message, from_address);

        // Recipient
        for to_address in to_addresses.iter() {
            try_smtp!(self.rcpt(to_address.as_slice()), self);
            // Log the rcpt command
            info!("{}: to=<{}>", current_message, to_address);
        }

        // Data
        try_smtp!(self.data(), self);

        // Message content
        let result = self.message(message.as_slice());

        if result.is_ok() {
            // Increment the connection reuse counter
            self.state.connection_reuse_count = self.state.connection_reuse_count + 1;

            // Log the message
            info!("{}: conn_use={}, size={}, status=sent ({})", current_message,
                self.state.connection_reuse_count, message.len(), match result.as_ref().ok().unwrap().message().as_slice() {
                    [ref line, ..] => line.as_slice(),
                    [] => "no response",
                }
            );
        }

        // Test if we can reuse the existing connection
        if (!self.client_info.enable_connection_reuse) ||
            (self.state.connection_reuse_count >= self.client_info.connection_reuse_count_limit) {
            self.reset();
        }

        result
    }

    /// Connects to the configured server
    pub fn connect(&mut self) -> SmtpResult {
        // Connect should not be called when the client is already connected
        if self.stream.is_some() {
            close_and_return_err!("The connection is already established", self);
        }

        // Try to connect
        self.stream = Some(BufStream::new(try!(Connecter::connect(&self.client_info.server_addr))));

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
    pub fn helo(&mut self) -> SmtpResult {
        let hostname = self.client_info.hello_name.clone();
        let result = try!(self.command(format!("HELO {}", hostname).as_slice()));
        self.server_info = Some(
            ServerInfo{
                name: result.first_word().expect("Server announced no hostname"),
                esmtp_features: vec![],
            }
        );
        Ok(result)
    }

    /// Sends a EHLO command and fills `server_info`
    pub fn ehlo(&mut self) -> SmtpResult {
        let hostname = self.client_info.hello_name.clone();
        let result = try!(self.command(format!("EHLO {}", hostname).as_slice()));
        self.server_info = Some(
            ServerInfo{
                name: result.first_word().expect("Server announced no hostname"),
                esmtp_features: Extension::parse_esmtp_response(&result),
            }
        );
        Ok(result)
    }

    /// Sends a MAIL command
    pub fn mail(&mut self, address: &str) -> SmtpResult {
        // Checks message encoding according to the server's capability
        let options = match self.server_info.as_ref().unwrap().supports_feature(Extension::EightBitMime) {
            true => "BODY=8BITMIME",
            false => "",
        };

        self.command(format!("MAIL FROM:<{}> {}", address, options).as_slice())
    }

    /// Sends a RCPT command
    pub fn rcpt(&mut self, address: &str) -> SmtpResult {
        self.command(format!("RCPT TO:<{}>", address).as_slice())
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
            Some(ref argument) => self.command(format!("HELP {}", argument).as_slice()),
            None => self.command("HELP"),
        }
    }

    /// Sends a VRFY command
    pub fn vrfy(&mut self, address: &str) -> SmtpResult {
        self.command(format!("VRFY {}", address).as_slice())
    }

    /// Sends a EXPN command
    pub fn expn(&mut self, address: &str) -> SmtpResult {
        self.command(format!("EXPN {}", address).as_slice())
    }

    /// Sends a RSET command
    pub fn rset(&mut self) -> SmtpResult {
        self.command("RSET")
    }

    /// Sends an AUTH command with PLAIN mecanism
    pub fn auth_plain(&mut self, username: &str, password: &str) -> SmtpResult {
        let auth_string = format!("{}{}{}{}", NUL, username, NUL, password);
        self.command(format!("AUTH PLAIN {}", auth_string.as_bytes().to_base64(base64::STANDARD)).as_slice())
    }

    /// Sends an AUTH command with CRAM-MD5 mecanism
    pub fn auth_cram_md5(&mut self, username: &str, password: &str) -> SmtpResult {
        let encoded_challenge = try_smtp!(self.command("AUTH CRAM-MD5"), self).first_word().expect("No challenge");
        // TODO manage errors
        let challenge = encoded_challenge.from_base64().unwrap();

        let mut hmac = Hmac::new(Md5::new(), password.as_bytes());
        hmac.input(challenge.as_slice());

        let auth_string = format!("{} {}", username, hmac.result().code().to_hex());

        self.command(format!("AUTH CRAM-MD5 {}", auth_string.as_bytes().to_base64(base64::STANDARD)).as_slice())
    }

    /// Sends the message content and close
    pub fn message(&mut self, message_content: &str) -> SmtpResult {
        self.send_server(escape_dot(message_content).as_slice(), MESSAGE_ENDING)
    }

    /// Sends a string to the server and gets the response
    fn send_server(&mut self, string: &str, end: &str) -> SmtpResult {
        try!(write!(self.stream.as_mut().unwrap(), "{}{}", string, end));
        try!(self.stream.as_mut().unwrap().flush());

        debug!("Wrote: {}", escape_crlf(string));

        self.get_reply()
    }

    /// Gets the SMTP response
    fn get_reply(&mut self) -> SmtpResult {
        let mut line = String::new();
        try!(self.stream.as_mut().unwrap().read_line(&mut line));

        // If the string is too short to be a response code
        if line.len() < 3 {
            return Err(FromError::from_error("Could not parse reply code, line too short"));
        }

        let (severity, category, detail) =  match (line[0..1].parse::<Severity>(), line[1..2].parse::<Category>(), line[2..3].parse::<u8>()) {
            (Ok(severity), Ok(category), Ok(detail)) => (severity, category, detail),
            _ => return Err(FromError::from_error("Could not parse reply code")),
        };

        let mut message = Vec::new();

        // 3 chars for code + space + CRLF
        while line.len() > 6 {
            let end = line.len() - 2;
            message.push(line[4..end].to_string());
            if line.as_bytes()[3] == '-' as u8 {
                line.clear();
                try!(self.stream.as_mut().unwrap().read_line(&mut line));
            } else {
                line.clear();
            }
        }

        let response = Response::new(severity, category, detail, message);

        match response.is_positive() {
            true => Ok(response),
            false => Err(FromError::from_error(response)),
        }
    }
}
