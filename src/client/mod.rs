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
use std::io::net::tcp::TcpStream;
use std::io::net::ip::{SocketAddr, ToSocketAddr};

use tools::get_first_word;
use common::{CRLF, MESSAGE_ENDING};
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

/// Structure that implements the SMTP client
pub struct Client<S = TcpStream> {
    /// TCP stream between client and server
    /// Value is None before connection
    stream: Option<S>,
    /// Socket we are connecting to
    server_addr: SocketAddr,
    /// Our hostname for HELO/EHLO commands
    my_hostname: String,
    /// Information about the server
    /// Value is None before HELO/EHLO
    server_info: Option<ServerInfo>,
    /// Transaction state, to check the sequence of commands
    state: TransactionState,
}

macro_rules! try_smtp (
    ($err: expr $client: ident) => ({
        match $err {
            Ok(val) => val,
            Err(err) => fail_with_err!(err $client),
        }
    })
)

macro_rules! fail_with_err (
    ($err: expr $client: ident) => ({
        $client.close_on_error();
        return Err(FromError::from_error($err))
    })
)

macro_rules! check_command_sequence (
    ($command: ident $client: ident) => ({
        if !$client.state.is_allowed(&$command) {
            fail_with_err!(
                Response{code: 503, message: Some("Bad sequence of commands".to_string())} $client
            );
        }
    })
)

impl<S = TcpStream> Client<S> {
    /// Creates a new SMTP client
    ///
    /// It does not connects to the server, but only create the `Client`
    pub fn new<A: ToSocketAddr>(addr: A, my_hostname: Option<&str>) -> Client<S> {
        Client{
            stream: None,
            server_addr: addr.to_socket_addr().unwrap(),
            my_hostname: my_hostname.unwrap_or("localhost").to_string(),
            server_info: None,
            state: TransactionState::new(),
        }
    }

    /// Creates a new local SMTP client to port 25
    ///
    /// It does not connects to the server, but only create the `Client`
    pub fn localhost() -> Client<S> {
        Client::new("localhost", Some("localhost"))
    }
}

impl<S: Connecter + ClientStream + Clone = TcpStream> Client<S> {
    /// Closes the SMTP transaction if possible, and then closes the TCP session
    fn close_on_error(&mut self) {
        if self.is_connected() {
            let _ = self.quit();
        }
        self.close();
    }

    /// Sends an email
    pub fn send_email(&mut self, email: SendableEmail) -> SmtpResult {
        // Connect to the server
        try!(self.connect());

        // Extended Hello or Hello
        if let Err(error) = self.ehlo() {
            match error.kind {
                ErrorKind::PermanentError(Response{code: 550, message: _}) => {
                    try_smtp!(self.helo() self);
                },
                _ => {
                    try_smtp!(Err(error) self)
                },
            };
        }

        // Print server information
        debug!("server {}", self.server_info.as_ref().unwrap());

        // Mail
        try_smtp!(self.mail(email.from_address.as_slice()) self);

        // Log the mail command
        info!("from=<{}>, size={}, nrcpt={}", email.from_address, email.message.len(), email.to_addresses.len());

        // Recipient
        // TODO Return rejected addresses
        // TODO Manage the number of recipients
        for to_address in email.to_addresses.iter() {
            try_smtp!(self.rcpt(to_address.as_slice()) self);
        }

        // Data
        try_smtp!(self.data() self);

        // Message content
        let sent = try_smtp!(self.message(email.message.as_slice()) self);

        // Log the rcpt command
        info!("to=<{}>, status=sent ({})",
              email.to_addresses.connect(">, to=<"), sent);

        // Quit
        try_smtp!(self.quit() self);

        return Ok(sent);
    }

    /// Connects to the configured server
    pub fn connect(&mut self) -> SmtpResult {
        let command = Command::Connect;
        check_command_sequence!(command self);

        // Connect should not be called when the client is already connected
        if !self.stream.is_none() {
            fail_with_err!("The connection is already established" self);
        }

        // Try to connect
        self.stream = Some(try!(Connecter::connect(self.server_addr)));

        // Log the connection
        info!("connection established to {}",
              self.stream.as_mut().unwrap().peer_name().unwrap());

        let result = try!(self.stream.as_mut().unwrap().get_reply());

        let checked_result = try!(command.test_success(result));
        self.state = self.state.next_state(&command).unwrap();
        Ok(checked_result)
    }

    /// Sends an SMTP command
    pub fn send_command(&mut self, command: Command) -> SmtpResult {
        // for now we do not support SMTPUTF8
        if !command.is_ascii() {
            fail_with_err!("Non-ASCII string" self);
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
                                command.to_string().as_slice(), CRLF
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

    /// Closes the TCP stream
    pub fn close(&mut self) {
        // Close the TCP connection
        drop(self.stream.as_mut().unwrap());
        // Reset client state
        self.stream = None;
        self.state = TransactionState::new();
        self.server_info = None;
    }

    /// Send a HELO command and fills `server_info`
    pub fn helo(&mut self) -> SmtpResult {
        let hostname = self.my_hostname.clone();
        let result = try!(self.send_command(Command::Hello(hostname)));
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
        let hostname = self.my_hostname.clone();
        let result = try!(self.send_command(Command::ExtendedHello(hostname)));
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

        let server_info = match self.server_info.clone() {
            Some(info) => info,
            None       => fail_with_err!(Response{
                                    code: 503,
                                    message: Some("Bad sequence of commands".to_string()),
                          } self),
        };

        // Checks message encoding according to the server's capability
        // TODO : Add an encoding check.
        let options = match server_info.supports_feature(Extension::EightBitMime) {
            Some(extension) => Some(vec![extension.client_mail_option().unwrap().to_string()]),
            None => None,
        };

        self.send_command(
            Command::Mail(from_address.to_string(), options)
        )
    }

    /// Sends a RCPT command
    pub fn rcpt(&mut self, to_address: &str) -> SmtpResult {
        self.send_command(
            Command::Recipient(to_address.to_string(), None)
        )
    }

    /// Sends a DATA command
    pub fn data(&mut self) -> SmtpResult {
        self.send_command(Command::Data)
    }

    /// Sends the message content
    pub fn message(&mut self, message_content: &str) -> SmtpResult {

        let server_info = match self.server_info.clone() {
            Some(info) => info,
            None       => fail_with_err!(Response{
                                    code: 503,
                                    message: Some("Bad sequence of commands".to_string()),
                          } self)
        };

        // Check message encoding
        if !server_info.supports_feature(Extension::EightBitMime).is_some() {
            if !message_content.is_ascii() {
                fail_with_err!("Server does not accepts UTF-8 strings" self);
            }
        }

        // Get maximum message size if defined and compare to the message size
        if let Some(Extension::Size(max)) = server_info.supports_feature(Extension::Size(0)) {
            if message_content.len() > max {
                fail_with_err!(Response{
                    code: 552,
                    message: Some("Message exceeds fixed maximum message size".to_string()),
                } self);
            }
        }

        self.send_message(message_content)
    }

    /// Sends a QUIT command
    pub fn quit(&mut self) -> SmtpResult {
        self.send_command(Command::Quit)
    }

    /// Sends a RSET command
    pub fn rset(&mut self) -> SmtpResult {
        self.send_command(Command::Reset)
    }

    /// Sends a NOOP command
    pub fn noop(&mut self) -> SmtpResult {
        self.send_command(Command::Noop)
    }

    /// Sends a VRFY command
    pub fn vrfy(&mut self, to_address: &str) -> SmtpResult {
        self.send_command(Command::Verify(to_address.to_string()))
    }

    /// Sends a EXPN command
    pub fn expn(&mut self, list: &str) -> SmtpResult {
        self.send_command(Command::Expand(list.to_string()))
    }

}
