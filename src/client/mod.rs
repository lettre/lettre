// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP client

use std::slice::Iter;
use std::string::String;
use std::error::FromError;
use std::old_io::net::tcp::TcpStream;
use std::old_io::net::ip::{SocketAddr, ToSocketAddr};

use uuid::Uuid;
use serialize::base64::{self, ToBase64, FromBase64};
use serialize::hex::ToHex;
use crypto::hmac::Hmac;
use crypto::md5::Md5;
use crypto::mac::Mac;

use SMTP_PORT;
use tools::get_first_word;
use tools::{NUL, CRLF, MESSAGE_ENDING};
use response::Response;
use extension::Extension;
use error::{SmtpResult, ErrorKind};
use sendable_email::SendableEmail;
use client::connecter::Connecter;
use client::server_info::ServerInfo;
use client::stream::ClientStream;

pub mod server_info;
pub mod connecter;
pub mod stream;

/// Represents the configuration of a client
#[derive(Debug)]
pub struct Configuration {
    /// Maximum connection reuse
    ///
    /// Zero means no limitation
    pub connection_reuse_count_limit: u16,
    /// Enable connection reuse
    pub enable_connection_reuse: bool,
    /// Maximum line length
    pub line_length_limit: u16,
    /// Name sent during HELO or EHLO
    pub hello_name: String,
}

/// Represents the state of a client
#[derive(Debug)]
pub struct State {
    /// Panic state
    pub panic: bool,
    /// Connection reuse counter
    pub connection_reuse_count: u16,
}

/// Represents the credentials
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

/// Structure that implements the SMTP client
pub struct Client<S = TcpStream> {
    /// TCP stream between client and server
    /// Value is None before connection
    stream: Option<S>,
    /// Socket we are connecting to
    server_addr: SocketAddr,
    /// Information about the server
    /// Value is None before HELO/EHLO
    server_info: Option<ServerInfo>,
    /// Client variable states
    state: State,
    /// Configuration of the client
    configuration: Configuration,
    /// Client credentials
    credentials: Option<Credentials>,
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

macro_rules! with_code (
    ($result: ident, $codes: expr) => ({
        match $result {
            Ok(response) => {
                for code in $codes {
                    if *code == response.code {
                        return Ok(response);
                    }
                }
                Err(FromError::from_error(response))
            },
            Err(_) => $result,
        }
    })
);

impl<S = TcpStream> Client<S> {
    /// Creates a new SMTP client
    ///
    /// It does not connects to the server, but only creates the `Client`
    pub fn new<A: ToSocketAddr>(addr: A) -> Client<S> {
        Client{
            stream: None,
            server_addr: addr.to_socket_addr().ok().expect("could not parse server address"),
            server_info: None,
            configuration: Configuration {
                connection_reuse_count_limit: 100,
                enable_connection_reuse: false,
                line_length_limit: 998,
                hello_name: "localhost".to_string(),
            },
            state: State {
                panic: false,
                connection_reuse_count: 0,
            },
            credentials: None,
        }
    }

    /// Creates a new local SMTP client to port 25
    ///
    /// It does not connects to the server, but only creates the `Client`
    pub fn localhost() -> Client<S> {
        Client::new(("localhost", SMTP_PORT))
    }

    /// Set the name used during HELO or EHLO
    pub fn set_hello_name(&mut self, name: &str) {
        self.configuration.hello_name = name.to_string()
    }

    /// Set the maximum number of emails sent using one connection
    pub fn set_enable_connection_reuse(&mut self, enable: bool) {
        self.configuration.enable_connection_reuse = enable
    }

    /// Set the maximum number of emails sent using one connection
    pub fn set_connection_reuse_count_limit(&mut self, count: u16) {
        self.configuration.connection_reuse_count_limit = count
    }

    /// Set the client credentials
    pub fn set_credentials(&mut self, username: &str, password: &str) {
        self.credentials = Some(Credentials {
            username: username.to_string(),
            password: password.to_string(),
        })
    }
}

impl<S: Connecter + ClientStream + Clone = TcpStream> Client<S> {
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
            info!("connection established to {}",
                self.stream.as_mut().unwrap().peer_name().unwrap());

            // Extended Hello or Hello if needed
            if let Err(error) = self.ehlo() {
                match error.kind {
                    ErrorKind::PermanentError(Response{code: 550, message: _}) => {
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
        if self.credentials.is_some() && self.state.connection_reuse_count == 0 {
            let credentials = self.credentials.clone().unwrap();
            if self.server_info.as_ref().unwrap().supports_feature(Extension::CramMd5Authentication).is_some() {
                let result = self.auth_cram_md5(credentials.username.as_slice(),
                                                credentials.password.as_slice());
                try_smtp!(result, self);
            } else if self.server_info.as_ref().unwrap().supports_feature(Extension::PlainAuthentication).is_some() {
                let result = self.auth_plain(credentials.username.as_slice(),
                                             credentials.password.as_slice());
                try_smtp!(result, self);
            } else {
                debug!("No supported authentication mecanisms available");
            }
        }

        let current_message = Uuid::new_v4();
        email.set_message_id(format!("<{}@{}>", current_message,
            self.configuration.hello_name.clone()));

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
                self.state.connection_reuse_count, message.len(), result.as_ref().ok().unwrap());
        }

        // Test if we can reuse the existing connection
        if (!self.configuration.enable_connection_reuse) ||
            (self.state.connection_reuse_count >= self.configuration.connection_reuse_count_limit) {
            self.reset();
        }

        result
    }

    /// Connects to the configured server
    fn connect(&mut self) -> SmtpResult {
        // Connect should not be called when the client is already connected
        if self.stream.is_some() {
            close_and_return_err!("The connection is already established", self);
        }

        // Try to connect
        self.stream = Some(try!(Connecter::connect(self.server_addr)));

        let result = self.stream.as_mut().unwrap().get_reply();
        with_code!(result, [220].iter())
    }

    /// Sends content to the server
    fn send_server(&mut self, content: &str, end: &str, expected_codes: Iter<u16>) -> SmtpResult {
        let result = self.stream.as_mut().unwrap().send_and_get_response(content, end);
        with_code!(result, expected_codes)
    }

    /// Checks if the server is connected using the NOOP SMTP command
    fn is_connected(&mut self) -> bool {
        self.noop().is_ok()
    }

    /// Sends an SMTP command
    fn command(&mut self, command: &str, expected_codes: Iter<u16>) -> SmtpResult {
        self.send_server(command, CRLF, expected_codes)
    }

    /// Send a HELO command and fills `server_info`
    fn helo(&mut self) -> SmtpResult {
        let hostname = self.configuration.hello_name.clone();
        let result = try!(self.command(format!("HELO {}", hostname).as_slice(), [250].iter()));
        self.server_info = Some(
            ServerInfo{
                name: get_first_word(result.message.as_ref().unwrap().as_slice()).to_string(),
                esmtp_features: vec!(),
            }
        );
        Ok(result)
    }

    /// Sends a EHLO command and fills `server_info`
    fn ehlo(&mut self) -> SmtpResult {
        let hostname = self.configuration.hello_name.clone();
        let result = try!(self.command(format!("EHLO {}", hostname).as_slice(), [250].iter()));
        self.server_info = Some(
            ServerInfo{
                name: get_first_word(result.message.as_ref().unwrap().as_slice()).to_string(),
                esmtp_features: Extension::parse_esmtp_response(
                                    result.message.as_ref().unwrap().as_slice()
                                ),
            }
        );
        Ok(result)
    }

    /// Sends a MAIL command
    fn mail(&mut self, address: &str) -> SmtpResult {
        // Checks message encoding according to the server's capability
        let options = match self.server_info.as_ref().unwrap().supports_feature(Extension::EightBitMime) {
            Some(_) => "BODY=8BITMIME",
            None => "",
        };

        self.command(format!("MAIL FROM:<{}> {}", address, options).as_slice(), [250].iter())
    }

    /// Sends a RCPT command
    fn rcpt(&mut self, address: &str) -> SmtpResult {
        self.command(format!("RCPT TO:<{}>", address).as_slice(), [250, 251].iter())
    }

    /// Sends a DATA command
    fn data(&mut self) -> SmtpResult {
        self.command("DATA", [354].iter())
    }

    /// Sends a QUIT command
    fn quit(&mut self) -> SmtpResult {
        self.command("QUIT", [221].iter())
    }

    /// Sends a NOOP command
    fn noop(&mut self) -> SmtpResult {
        self.command("NOOP", [250].iter())
    }

    /// Sends an AUTH command with PLAIN mecanism
    fn auth_plain(&mut self, username: &str, password: &str) -> SmtpResult {
        let auth_string = format!("{}{}{}{}{}", "", NUL, username, NUL, password);
        self.command(format!("AUTH PLAIN {}", auth_string.as_bytes().to_base64(base64::STANDARD)).as_slice(), [235].iter())
    }

    /// Sends an AUTH command with CRAM-MD5 mecanism
    fn auth_cram_md5(&mut self, username: &str, password: &str) -> SmtpResult {
        let encoded_challenge = try_smtp!(self.command("AUTH CRAM-MD5", [334].iter()), self).message.unwrap();
        // TODO manage errors
        let challenge = encoded_challenge.from_base64().unwrap();

        let mut hmac = Hmac::new(Md5::new(), password.as_bytes());
        hmac.input(challenge.as_slice());

        let auth_string = format!("{} {}", username, hmac.result().code().to_hex());

        self.command(format!("AUTH CRAM-MD5 {}", auth_string.as_bytes().to_base64(base64::STANDARD)).as_slice(), [235].iter())
    }

    /// Sends the message content and close
    fn message(&mut self, message_content: &str) -> SmtpResult {
        self.send_server(message_content, MESSAGE_ENDING, [250].iter())
    }
}
