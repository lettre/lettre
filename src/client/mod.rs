// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP client

use std::ascii::AsciiExt;
use std::string::String;
use std::error::FromError;
use std::io::net::tcp::TcpStream;
use std::io::net::ip::{SocketAddr, ToSocketAddr};

use uuid::Uuid;

use tools::get_first_word;
use common::{CRLF, MESSAGE_ENDING, SMTP_PORT};
use response::Response;
use extension::Extension;
use command::Command;
use transaction::TransactionState;
use error::{SmtpResult, ErrorKind};
use client::connecter::Connecter;
use client::server_info::ServerInfo;
use client::stream::ClientStream;
use sendable_email::SendableEmail;

pub mod server_info;
pub mod connecter;
pub mod stream;

/// Represents the configuration of a client
#[derive(Clone)]
pub struct Configuration {
    /// Maximum connection reuse
    ///
    /// Zero means no limitation
    pub connection_reuse_count_limit: usize,
    /// Enable connection reuse
    pub enable_connection_reuse: bool,
    /// Maximum recipients
    pub destination_recipient_limit: usize,
    /// Maximum line length
    pub line_length_limit: usize,
    /// Name sent during HELO or EHLO
    pub hello_name: String,
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
    /// Transaction state, to check the sequence of commands
    state: TransactionState,
    /// Panic state
    panic: bool,
    /// Connection reuse counter
    connection_reuse_count: usize,
    /// Current message id
    current_message: Option<Uuid>,
    /// Configuration of the client
    configuration: Configuration,
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
        if !$client.panic {
            $client.panic = true;
            $client.close();
        }
        return Err(FromError::from_error($err))
    })
);

macro_rules! check_command_sequence (
    ($command: ident $client: ident) => ({
        if !$client.state.is_allowed(&$command) {
            close_and_return_err!(
                Response{code: 503, message: Some("Bad sequence of commands".to_string())}, $client
            );
        }
    })
);

impl<S = TcpStream> Client<S> {
    /// Creates a new SMTP client
    ///
    /// It does not connects to the server, but only create the `Client`
    pub fn new<A: ToSocketAddr>(addr: A) -> Client<S> {
        Client{
            stream: None,
            server_addr: addr.to_socket_addr().unwrap(),
            server_info: None,
            state: TransactionState::new(),
            panic: false,
            connection_reuse_count: 0,
            current_message: None,
            configuration: Configuration {
                connection_reuse_count_limit: 100,
                enable_connection_reuse: false,
                line_length_limit: 998,
                destination_recipient_limit: 100,
                hello_name: "localhost".to_string(),
            }
        }
    }

    /// Creates a new local SMTP client to port 25
    ///
    /// It does not connects to the server, but only create the `Client`
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
    pub fn set_connection_reuse_count_limit(&mut self, count: usize) {
        self.configuration.connection_reuse_count_limit = count
    }

    /// Set the client configuration
    pub fn set_configuration(&mut self, configuration: Configuration) {
        self.configuration = configuration
    }
}

impl<S: Connecter + ClientStream + Clone = TcpStream> Client<S> {
    /// Closes the SMTP transaction if possible
    pub fn close(&mut self) {
        let _ = self.quit();
    }

    /// Reset the client state
    pub fn reset(&mut self) {
        // Close the SMTP transaction if needed
        self.close();

        // Reset the client state
        self.stream = None;
        self.state = TransactionState::new();
        self.server_info = None;
        self.panic = false;
        self.connection_reuse_count = 0;
        self.current_message = None;
    }

    /// Sends an email
    pub fn send<T: SendableEmail>(&mut self, mut email: T) -> SmtpResult {

        // If there is a usable connection, test if the server answers and hello has been sent
        if self.connection_reuse_count > 0 {
            if self.noop().is_err() {
                self.reset();
            }
        }

        // Connect to the server if needed
        if self.stream.is_none() || self.server_info.is_none() {
            try!(self.connect());

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
            //debug!("server {}", self.server_info.as_ref().unwrap());
        }

        self.current_message = Some(Uuid::new_v4());
        email.set_message_id(format!("<{}@{}>", self.current_message.as_ref().unwrap(),
            self.configuration.hello_name.clone()));

        let from_address = email.from_address();
        let to_addresses = email.to_addresses();
        let message = email.message();

        // Mail
        try_smtp!(self.mail(from_address.as_slice()), self);

        // Recipient
        // TODO Return rejected addresses
        // TODO Limit the number of recipients
        for to_address in to_addresses.iter() {
            try_smtp!(self.rcpt(to_address.as_slice()), self);
        }

        // Data
        try_smtp!(self.data(), self);

        // Message content
        self.message(message.as_slice())
    }

    /// Connects to the configured server
    pub fn connect(&mut self) -> SmtpResult {
        let command = Command::Connect;

        check_command_sequence!(command self);

        // Connect should not be called when the client is already connected
        if self.stream.is_some() {
            close_and_return_err!("The connection is already established", self);
        }

        // Try to connect
        self.stream = Some(try!(Connecter::connect(self.server_addr)));

        // Log the connection
        //info!("connection established to {}",
        //      self.stream.as_mut().unwrap().peer_name().unwrap());

        let result = try!(self.stream.as_mut().unwrap().get_reply());

        let checked_result = try!(command.test_success(result));
        self.state = self.state.next_state(&command).unwrap();
        Ok(checked_result)
    }

    /// Sends an SMTP command
    fn command(&mut self, command: Command) -> SmtpResult {
        // for now we do not support SMTPUTF8
        if !command.is_ascii() {
            close_and_return_err!("Non-ASCII string", self);
        }

        self.send_server(command, None)
    }

    /// Sends the email content
    fn send_message(&mut self, message: &str) -> SmtpResult {
        self.send_server(Command::Message, Some(message))
    }

    /// Sends content to the server, after checking the command sequence, and then
    /// updates the transaction state
    ///
    /// * If `message` is `None`, the given command will be formatted and sent to the server
    /// * If `message` is `Some(str)`, the `str` string will be sent to the server
    fn send_server(&mut self, command: Command, message: Option<&str>) -> SmtpResult {
        check_command_sequence!(command self);

        let result = try!(match message {
            Some(message) => self.stream.as_mut().unwrap().send_and_get_response(
                                message, MESSAGE_ENDING
                             ),
            None          => self.stream.as_mut().unwrap().send_and_get_response(
                                format! ("{:?}", command) .as_slice(), CRLF
                             ),
        });

        let checked_result = try!(command.test_success(result));
        self.state = self.state.next_state(&command).unwrap();
        Ok(checked_result)
    }

    /// Checks if the server is connected using the NOOP SMTP command
    pub fn is_connected(&mut self) -> bool {
        self.noop().is_ok()
    }

    /// Send a HELO command and fills `server_info`
    pub fn helo(&mut self) -> SmtpResult {
        let hostname = self.configuration.hello_name.clone();
        let result = try!(self.command(Command::Hello(hostname)));
        self.server_info = Some(
            ServerInfo{
                name: get_first_word(result.message.as_ref().unwrap().as_slice()).to_string(),
                esmtp_features: None,
            }
        );
        Ok(result)
    }

    /// Sends a EHLO command and fills `server_info`
    pub fn ehlo(&mut self) -> SmtpResult {
        let hostname = self.configuration.hello_name.clone();
        let result = try!(self.command(Command::ExtendedHello(hostname)));
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
    pub fn mail(&mut self, from_address: &str) -> SmtpResult {

        // Generate an ID for the logs if None was provided
        if self.current_message.is_none() {
            self.current_message = Some(Uuid::new_v4());
        }

        let server_info = match self.server_info.clone() {
            Some(info) => info,
            None       => close_and_return_err!(Response{
                                    code: 503,
                                    message: Some("Bad sequence of commands".to_string()),
                          }, self),
        };

        // Checks message encoding according to the server's capability
        // TODO : Add an encoding check.
        let options = match server_info.supports_feature(Extension::EightBitMime) {
            Some(extension) => Some(vec![extension.client_mail_option().unwrap().to_string()]),
            None => None,
        };

        let result = self.command(
            Command::Mail(from_address.to_string(), options)
        );

        if result.is_ok() {
            // Log the mail command
            //info!("{}: from=<{}>", self.current_message.as_ref().unwrap(), from_address);
        }

        result
    }

    /// Sends a RCPT command
    pub fn rcpt(&mut self, to_address: &str) -> SmtpResult {
        let result = self.command(
            Command::Recipient(to_address.to_string(), None)
        );

        if result.is_ok() {
            // Log the rcpt command
            //info!("{}: to=<{}>", self.current_message.as_ref().unwrap(), to_address);
        }

        result
    }

    /// Sends a DATA command
    pub fn data(&mut self) -> SmtpResult {
        self.command(Command::Data)
    }

    /// Sends the message content
    pub fn message(&mut self, message_content: &str) -> SmtpResult {

        let server_info = match self.server_info.clone() {
            Some(info) => info,
            None       => close_and_return_err!(Response{
                                    code: 503,
                                    message: Some("Bad sequence of commands".to_string()),
                          }, self)
        };

        // Check message encoding
        if !server_info.supports_feature(Extension::EightBitMime).is_some() {
            if !message_content.as_bytes().is_ascii() {
                close_and_return_err!("Server does not accepts UTF-8 strings", self);
            }
        }

        // Get maximum message size if defined and compare to the message size
        if let Some(Extension::Size(max)) = server_info.supports_feature(Extension::Size(0)) {
            if message_content.len() > max {
                close_and_return_err!(Response{
                    code: 552,
                    message: Some("Message exceeds fixed maximum message size".to_string()),
                }, self);
            }
        }

        let result = self.send_message(message_content);

        if result.is_ok() {
            // Increment the connection reuse counter
            self.connection_reuse_count = self.connection_reuse_count + 1;
            // Log the message
            //info!("{}: conn_use={}, size={}, status=sent ({})", self.current_message.as_ref().unwrap(),
            //    self.connection_reuse_count, message_content.len(), result.as_ref().ok().unwrap());
        }

        self.current_message = None;

        // Test if we can reuse the existing connection
        if (!self.configuration.enable_connection_reuse) ||
            (self.connection_reuse_count == self.configuration.connection_reuse_count_limit) {
            self.reset();
        }

        result
    }

    /// Sends a QUIT command
    pub fn quit(&mut self) -> SmtpResult {
        self.command(Command::Quit)
    }

    /// Sends a RSET command
    pub fn rset(&mut self) -> SmtpResult {
        self.command(Command::Reset)
    }

    /// Sends a NOOP command
    pub fn noop(&mut self) -> SmtpResult {
        self.command(Command::Noop)
    }

    /// Sends a VRFY command
    pub fn vrfy(&mut self, to_address: &str) -> SmtpResult {
        self.command(Command::Verify(to_address.to_string()))
    }

    /// Sends a EXPN command
    pub fn expn(&mut self, list: &str) -> SmtpResult {
        self.command(Command::Expand(list.to_string()))
    }

}
