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
use std::io::net::ip::Port;
use std::error::FromError;

use common::{get_first_word, unquote_email_address};
use common::{CRLF, SMTP_PORT, MESSAGE_ENDING};
use response::Response;
use extension;
use extension::Extension;
use command;
use command::Command;
use transaction::TransactionState;
use error::{SmtpResult, ErrorKind};
use client::connecter::Connecter;
use client::server_info::ServerInfo;
use client::stream::ClientStream;

pub mod server_info;
mod connecter;
mod stream;

/// Structure that implements the SMTP client
pub struct Client<S> {
    /// TCP stream between client and server
    /// Value is None before connection
    stream: Option<S>,
    /// Host we are connecting to
    host: String,
    /// Port we are connecting on
    port: Port,
    /// Our hostname for HELO/EHLO commands
    my_hostname: String,
    /// Information about the server
    /// Value is None before HELO/EHLO
    server_info: Option<ServerInfo>,
    /// Transaction state, to check the sequence of commands
    state: TransactionState
}

macro_rules! try_smtp (
    ($expr:expr $sp: ident) => ({
        match $expr {
            Ok(val) => val,
            Err(err) => fail_with_err!(err $sp)
        }
    })
)

macro_rules! fail_with_err (
    ($expr:expr $sp: ident) => ({
        $sp.close_on_error::<S>();
        return Err(FromError::from_error($expr))
    })
)

impl<S> Client<S> {
    /// Creates a new SMTP client
    ///
    /// It does not connects to the server, but only create the `Client`
    pub fn new(host: Option<&str>, port: Option<Port>, my_hostname: Option<&str>) -> Client<S> {
        Client{
            stream: None,
            host: host.unwrap_or("localhost").to_string(),
            port: port.unwrap_or(SMTP_PORT),
            my_hostname: my_hostname.unwrap_or("localhost").to_string(),
            server_info: None,
            state: TransactionState::new()
        }
    }
}

impl<S: Connecter + ClientStream + Clone> Client<S> {
    /// Closes the SMTP transaction if possible, and then closes the TCP session
    fn close_on_error<S>(&mut self) {
        if self.is_connected::<S>() {
            let _ = self.quit::<S>();
        }
        self.close();
    }

    /// Sends an email
    pub fn send_mail<S>(&mut self, from_address: &str,
                        to_addresses: Vec<&str>, message: &str) -> SmtpResult {
        let my_hostname = self.my_hostname.clone();

        // Connect to the server
        try!(self.connect());

        // Extended Hello or Hello
        match self.ehlo::<S>(my_hostname.as_slice()) {
            Err(error) => match error.kind {
                            ErrorKind::PermanentError(Response{code: 550, message: _}) => {
                                try_smtp!(self.helo::<S>(my_hostname.as_slice()) self);
                            },
                            _ => {
                                try_smtp!(Err(error) self)
                            },
                          },
            _ => {}
        }

        debug!("Server {}", self.server_info.clone().unwrap());

        // Mail
        try_smtp!(self.mail::<S>(from_address) self);

        // Log the mail command
        info!("from=<{}>, size={}, nrcpt={}", from_address, message.len(), to_addresses.len());

        // Recipient
        // TODO Return rejected addresses
        // TODO Manage the number of recipients
        for to_address in to_addresses.iter() {
            try_smtp!(self.rcpt::<S>(*to_address) self);
        }

        // Data
        try_smtp!(self.data::<S>() self);

        // Message content
        let sent = try_smtp!(self.message::<S>(message) self);

        info!("to=<{}>, status=sent ({})",
              to_addresses.connect(">, to=<"), sent.clone());

        // Quit
        try_smtp!(self.quit::<S>() self);

        return Ok(sent);
    }

    /// Sends an SMTP command
    fn send_command(&mut self, command: Command) -> SmtpResult {
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
        if !self.state.is_command_possible(command.clone()) {
            fail_with_err!(Response{code: 503, message: Some("Bad sequence of commands".to_string())} self);
        }

        let result = try!(match message {
            Some(message) => self.stream.clone().unwrap().send_and_get_response(
                                message, MESSAGE_ENDING
                             ),
            None          => self.stream.clone().unwrap().send_and_get_response(
                                command.to_string().as_slice(), CRLF
                             )
        });

        let checked_result = try!(command.clone().test_success(result));

        self.state = self.state.next_state(command.clone()).unwrap();

        Ok(checked_result)
    }

    /// Connects to the configured server
    pub fn connect(&mut self) -> SmtpResult {
        let command = command::Connect;

        // connect should not be called when the client is already connected
        if !self.stream.is_none() {
            fail_with_err!("The connection is already established" self);
        }

        // Try to connect
        self.stream = Some(try!(Connecter::connect(self.host.clone().as_slice(), self.port)));

        // Log the connection
        info!("Connection established to {}[{}]:{}",
              self.host, self.stream.clone().unwrap().peer_name().unwrap().ip, self.port);

        let result = try!(self.stream.clone().unwrap().get_reply());

        let checked_result = try!(command.test_success(result));

        self.state = self.state.next_state(command).unwrap();
        Ok(checked_result)
    }

    /// Checks if the server is connected using the NOOP SMTP command
    pub fn is_connected<S>(&mut self) -> bool {
        self.noop::<S>().is_ok()
    }

    /// Closes the TCP stream
    pub fn close(&mut self) {
        // Close the TCP connection
        drop(self.stream.clone().unwrap());
        // Reset client state
        self.stream = None;
        self.state = TransactionState::new();
        self.server_info = None;
    }

    /// Send a HELO command and fills `server_info`
    pub fn helo<S>(&mut self, my_hostname: &str) -> SmtpResult {
        let result = try!(self.send_command(command::Hello(my_hostname.to_string())));
        self.server_info = Some(
            ServerInfo{
                name: get_first_word(result.message.clone().unwrap().as_slice()).to_string(),
                esmtp_features: None
            }
        );
        Ok(result)
    }

    /// Sends a EHLO command and fills `server_info`
    pub fn ehlo<S>(&mut self, my_hostname: &str) -> SmtpResult {
        let result = try!(self.send_command(command::ExtendedHello(my_hostname.to_string())));
        self.server_info = Some(
            ServerInfo{
                name: get_first_word(result.message.clone().unwrap().as_slice()).to_string(),
                esmtp_features: Extension::parse_esmtp_response(
                                    result.message.clone().unwrap().as_slice()
                                )
            }
        );
        Ok(result)
    }

    /// Sends a MAIL command
    pub fn mail<S>(&mut self, from_address: &str) -> SmtpResult {

        let server_info = match self.server_info.clone() {
            Some(info) => info,
            None       => fail_with_err!(Response{
                                    code: 503,
                                    message: Some("Bad sequence of commands".to_string())
                          } self)
        };

        // Checks message encoding according to the server's capability
        // TODO : Add an encoding check.
        let options = match server_info.supports_feature(extension::EightBitMime) {
            Some(extension) => Some(vec![extension.client_mail_option().unwrap().to_string()]),
            None => None
        };

        self.send_command(
            command::Mail(unquote_email_address(from_address).to_string(), options)
        )
    }

    /// Sends a RCPT command
    pub fn rcpt<S>(&mut self, to_address: &str) -> SmtpResult {
        self.send_command(
            command::Recipient(unquote_email_address(to_address).to_string(), None)
        )
    }

    /// Sends a DATA command
    pub fn data<S>(&mut self) -> SmtpResult {
        self.send_command(command::Data)
    }

    /// Sends the message content
    pub fn message<S>(&mut self, message_content: &str) -> SmtpResult {

        let server_info = match self.server_info.clone() {
            Some(info) => info,
            None       => fail_with_err!(Response{
                                    code: 503,
                                    message: Some("Bad sequence of commands".to_string())
                          } self)
        };

        // Check message encoding
        if !server_info.supports_feature(extension::EightBitMime).is_some() {
            if !message_content.clone().is_ascii() {
                fail_with_err!("Server does not accepts UTF-8 strings" self);
            }
        }

        // Get maximum message size if defined and compare to the message size
        match server_info.supports_feature(extension::Size(0)) {
            Some(extension::Size(max)) if message_content.len() > max =>
                fail_with_err!(Response{
                    code: 552,
                    message: Some("Message exceeds fixed maximum message size".to_string())
                } self),
            _ => ()
        };

        self.send_message(message_content)
    }

    /// Sends a QUIT command
    pub fn quit<S>(&mut self) -> SmtpResult {
        self.send_command(command::Quit)
    }

    /// Sends a RSET command
    pub fn rset<S>(&mut self) -> SmtpResult {
        self.send_command(command::Reset)
    }

    /// Sends a NOOP command
    pub fn noop<S>(&mut self) -> SmtpResult {
        self.send_command(command::Noop)
    }

    /// Sends a VRFY command
    pub fn vrfy<S>(&mut self, to_address: &str) -> SmtpResult {
        self.send_command(command::Verify(to_address.to_string()))
    }

    /// Sends a EXPN command
    pub fn expn<S>(&mut self, list: &str) -> SmtpResult {
        self.send_command(command::Expand(list.to_string()))
    }

}
