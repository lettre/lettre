// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP client

use std::fmt::Show;
use std::result::Result;
use std::string::String;
use std::io::net::ip::Port;

use common::{get_first_word, unquote_email_address};
use response::Response;
use extension;
use command;
use command::Command;
use common::{SMTP_PORT, CRLF};
use transaction;
use transaction::TransactionState;
use client::connecter::Connecter;
use client::server_info::ServerInfo;
use client::stream::ClientStream;

mod connecter;
mod server_info;
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

impl<S> Client<S> {
    /// Creates a new SMTP client
    pub fn new(host: String, port: Option<Port>, my_hostname: Option<String>) -> Client<S> {
        Client{
            stream: None,
            host: host,
            port: port.unwrap_or(SMTP_PORT),
            my_hostname: my_hostname.unwrap_or(String::from_str("localhost")),
            server_info: None,
            state: transaction::Unconnected
        }
    }
}

// T : String ou String, selon le support
impl<S: Connecter + ClientStream + Clone> Client<S> {

    /// TODO
    fn smtp_fail_if_err<S>(&mut self, response: Result<Response, Response>) {
        match response {
            Err(response) => {
                self.smtp_fail::<S, Response>(response)
            },
            Ok(_) => {}
        }
    }

    /// Closes the connection and fail with a given messgage
    fn smtp_fail<S, T: Show>(&mut self, reason: T) {
        let is_connected = self.is_connected::<S>();
        if is_connected {
            match self.quit::<S>() {
                Ok(..) => {},
                Err(response) => panic!("Failed: {}", response)
            }
        }
        self.close();
        panic!("Failed: {}", reason);
    }

    /// Sends an email
    pub fn send_mail<S>(&mut self, from_address: String,
                        to_addresses: Vec<String>, message: String) {
        let my_hostname = self.my_hostname.clone();
        let mut smtp_result: Result<Response, Response>;

        match self.connect() {
            Ok(_) => {},
            Err(response) => panic!("Cannot connect to {:s}:{:u}. Server says: {}",
                                    self.host,
                                    self.port, response
                             )
        }

        // Extended Hello or Hello
        match self.ehlo::<S>(my_hostname.clone().to_string()) {
            Err(Response{code: 550, message: _}) => {
                smtp_result = self.helo::<S>(my_hostname.clone());
                self.smtp_fail_if_err::<S>(smtp_result);
            },
            Err(response) => {
                self.smtp_fail::<S, Response>(response)
            }
            _ => {}
        }

        debug!("Server {}", self.server_info.clone().unwrap());

        // Checks message encoding according to the server's capability
        // TODO : Add an encoding check.
        if ! self.server_info.clone().unwrap().supports_feature(extension::EightBitMime).is_some() {
            if ! message.clone().is_ascii() {
                self.smtp_fail::<S, &str>("Server does not accepts UTF-8 strings");
            }
        }

        // Get maximum message size if defined
        let max_size = match self.server_info.clone().unwrap().supports_feature(extension::Size(0)) {
            Some(extension::Size(max)) => max,
            _ => -1
        };

        // Check maximum message size
        if max_size > 0 && message.len() > max_size {
            self.smtp_fail::<S, &str>("Message is too big. The limit is {}");
        }

        // Mail
        smtp_result = self.mail::<S>(from_address.clone(), None);
        self.smtp_fail_if_err::<S>(smtp_result);

        // Log the mail command
        info!("from=<{}>, size={}, nrcpt={}", from_address, message.len(), to_addresses.len());

        // Recipient
        // TODO Return rejected addresses
        // TODO Manage the number of recipients
        for to_address in to_addresses.iter() {
            smtp_result = self.rcpt::<S>(to_address.clone(), None);
            self.smtp_fail_if_err::<S>(smtp_result);
        }

        // Data
        smtp_result = self.data::<S>();
        self.smtp_fail_if_err::<S>(smtp_result);

        // Message content
        let sent = self.message::<S>(message);

        if sent.clone().is_err() {
            self.smtp_fail::<S, Response>(sent.clone().err().unwrap())
        }

        info!("to=<{}>, status=sent ({})",
              to_addresses.clone().connect(">, to=<"), sent.clone().ok().unwrap());

        // Quit
        smtp_result = self.quit::<S>();
        self.smtp_fail_if_err::<S>(smtp_result);
    }

    /// Sends an SMTP command
    // TODO : ensure this is an ASCII string
    fn send_command(&mut self, command: Command) -> Response {
        if !self.state.is_command_possible(command.clone()) {
            panic!("Bad command sequence");
        }
        self.stream.clone().unwrap().send_and_get_response(format!("{}", command).as_slice())
    }

    /// Sends the email content
    fn send_message(&mut self, message: String) -> Response {
        self.stream.clone().unwrap().send_and_get_response(format!("{}{}.{}", message, CRLF, CRLF).as_slice())
    }

    /// Connects to the configured server
    pub fn connect(&mut self) -> Result<Response, Response> {
        // connect should not be called when the client is already connected
        if !self.stream.is_none() {
            panic!("The connection is already established");
        }

        // Try to connect
        self.stream = match Connecter::connect(self.host.clone().as_slice(), self.port) {
            Ok(stream) => Some(stream),
            Err(..) => panic!("Cannot connect to the server")
        };

        // Log the connection
        info!("Connection established to {}[{}]:{}",
              self.host, self.stream.clone().unwrap().peer_name().unwrap().ip, self.port);

        match self.stream.clone().unwrap().get_reply() {
            Some(response) => match response.with_code(vec!(220)) {
                                  Ok(response)  => {
                                      self.state = transaction::Connected;
                                      Ok(response)
                                  },
                                  Err(response) => {
                                      Err(response)
                                  }
                              },
            None           => panic!("No banner on {}", self.host)
        }
    }

    /// Checks if the server is connected
    pub fn is_connected<S>(&mut self) -> bool {
        self.noop::<S>().is_ok()
    }

    /// Closes the TCP stream
    pub fn close(&mut self) {
        // Close the TCP connection
        drop(self.stream.clone().unwrap());
        // Reset client state
        self.stream = None;
        self.state = transaction::Unconnected;
        self.server_info = None;
    }

    /// Send a HELO command
    pub fn helo<S>(&mut self, my_hostname: String) -> Result<Response, Response> {
        match self.send_command(command::Hello(my_hostname.clone())).with_code(vec!(250)) {
            Ok(response) => {
                self.server_info = Some(
                    ServerInfo{
                        name: get_first_word(response.message.clone().unwrap()),
                        esmtp_features: None
                    }
                );
                self.state = transaction::HelloSent;
                Ok(response)
            },
            Err(response) => Err(response)
        }
    }

    /// Sends a EHLO command
    pub fn ehlo<S>(&mut self, my_hostname: String) -> Result<Response, Response> {
        match self.send_command(command::ExtendedHello(my_hostname.clone())).with_code(vec!(250)) {
            Ok(response) => {
                self.server_info = Some(
                    ServerInfo{
                        name: get_first_word(response.message.clone().unwrap()),
                        esmtp_features: ServerInfo::parse_esmtp_response(
                                            response.message.clone().unwrap()
                                        )
                    }
                );
                self.state = transaction::HelloSent;
                Ok(response)
            },
            Err(response) => Err(response)
        }
    }

    /// Sends a MAIL command
    pub fn mail<S>(&mut self, from_address: String,
                   options: Option<Vec<String>>) -> Result<Response, Response> {
        match self.send_command(
            command::Mail(unquote_email_address(from_address.to_string()), options)
        ).with_code(vec!(250)) {
            Ok(response) => {
                self.state = transaction::MailSent;
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends a RCPT command
    pub fn rcpt<S>(&mut self, to_address: String,
                   options: Option<Vec<String>>) -> Result<Response, Response> {
        match self.send_command(
            command::Recipient(unquote_email_address(to_address.to_string()), options)
        ).with_code(vec!(250)) {
            Ok(response) => {
                self.state = transaction::RecipientSent;
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends a DATA command
    pub fn data<S>(&mut self) -> Result<Response, Response> {
        match self.send_command(command::Data).with_code(vec!(354)) {
            Ok(response) => {
                self.state = transaction::DataSent;
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends the message content
    pub fn message<S>(&mut self, message_content: String) -> Result<Response, Response> {
        match self.send_message(message_content).with_code(vec!(250)) {
            Ok(response) => {
                self.state = transaction::HelloSent;
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends a QUIT command
    pub fn quit<S>(&mut self) -> Result<Response, Response> {
        match self.send_command(command::Quit).with_code(vec!(221)) {
            Ok(response) => {
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends a RSET command
    pub fn rset<S>(&mut self) -> Result<Response, Response> {
        match self.send_command(command::Reset).with_code(vec!(250)) {
            Ok(response) => {
                if vec!(transaction::MailSent, transaction::RecipientSent,
                        transaction::DataSent).contains(&self.state) {
                    self.state = transaction::HelloSent;
                }
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends a NOOP commands
    pub fn noop<S>(&mut self) -> Result<Response, Response> {
        self.send_command(command::Noop).with_code(vec!(250))
    }

    /// Sends a VRFY command
    pub fn vrfy<S, T>(&mut self, to_address: String) -> Result<Response, Response> {
        self.send_command(command::Verify(to_address, None)).with_code(vec!(250))
    }
}
