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
use common::{CRLF, SMTP_PORT};
use response::Response;
use extension;
use extension::Extension;
use command;
use command::Command;
use transaction;
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
            Err(err) => {
                $sp.smtp_fail::<S>();
                return Err(::std::error::FromError::from_error(err))
            }
        }
    })
)

impl<S> Client<S> {
    /// Creates a new SMTP client
    pub fn new(host: String, port: Option<Port>, my_hostname: Option<String>) -> Client<S> {
        Client{
            stream: None,
            host: host,
            port: port.unwrap_or(SMTP_PORT),
            my_hostname: my_hostname.unwrap_or("localhost".to_string()),
            server_info: None,
            state: transaction::Unconnected
        }
    }
}

impl<S: Connecter + ClientStream + Clone> Client<S> {

    /// TODO
    // fn try_smtp<S, T>(&mut self, result: SmtpResult<T>) -> T {
    //     match result {
    //         Ok(val) => val,
    //         Err(err) => {
    //             self.smtp_fail::<S>();
    //             return Err(FromError::from_error(err));
    //         }
    //     }
    // }

    /// Closes the SMTP transaction if possible, and then closes the TCP session
    fn smtp_fail<S>(&mut self) {
        if self.is_connected::<S>() {
            let _ = self.quit::<S>();
            self.close();
        } else {
            self.close();
        };
    }

    /// Sends an email
    pub fn send_mail<S>(&mut self, from_address: String,
                        to_addresses: Vec<String>, message: String) -> SmtpResult<Response> {
        let my_hostname = self.my_hostname.clone();

        // Connect to the server
        try!(self.connect());

        // Extended Hello or Hello
        match self.ehlo::<S>(my_hostname.clone().to_string()) {
            Err(error) => match error.kind {
                            ErrorKind::PermanentError(Response{code: 550, message: _}) => {
                                try_smtp!(self.helo::<S>(my_hostname.clone()) self);
                                //self.smtp_fail_if_err::<S>(smtp_result);
                            },
                            _ => {
                                self.smtp_fail::<S>();
                                return Err(FromError::from_error(error))
                            }
                          },
            _ => {}
        }

        debug!("Server {}", self.server_info.clone().unwrap());

        // Mail
        try_smtp!(self.mail::<S>(from_address.clone(), None) self);
        //self.smtp_fail_if_err::<S>(smtp_result);

        // Log the mail command
        info!("from=<{}>, size={}, nrcpt={}", from_address, message.len(), to_addresses.len());

        // Recipient
        // TODO Return rejected addresses
        // TODO Manage the number of recipients
        for to_address in to_addresses.iter() {
            try_smtp!(self.rcpt::<S>(to_address.clone(), None) self);
            //self.smtp_fail_if_err::<S>(smtp_result);
        }

        // Data
        try_smtp!(self.data::<S>() self);
        //self.smtp_fail_if_err::<S>(smtp_result);

        // Message content
        let sent = try_smtp!(self.message::<S>(message.as_slice()) self);

        info!("to=<{}>, status=sent ({})",
              to_addresses.clone().connect(">, to=<"), sent.clone());

        // Quit
        try_smtp!(self.quit::<S>() self);
        //self.smtp_fail_if_err::<S>(smtp_result);
        return Ok(Response{code:100, message:None});
    }

    /// Sends an SMTP command
    // TODO : ensure this is an ASCII string
    fn send_command(&mut self, command: Command) -> SmtpResult<Response> {
        if !command.is_ascii() {
            panic!("Non-ASCII string: {}", command);
        }
        if !self.state.is_command_possible(command.clone()) {
            panic!("Bad command sequence");
        }
        self.stream.clone().unwrap().send_and_get_response(format!("{}", command).as_slice(),
                                                           format!("{}", CRLF).as_slice())
    }

    /// Sends the email content
    fn send_message(&mut self, message: &str) -> SmtpResult<Response> {
        self.stream.clone().unwrap().send_and_get_response(format!("{}", message).as_slice(),
                                                           format!("{}.{}", CRLF, CRLF).as_slice())
    }

    /// Connects to the configured server
    pub fn connect(&mut self) -> SmtpResult<Response> {
        // connect should not be called when the client is already connected
        if !self.stream.is_none() {
            panic!("The connection is already established");
        }

        // Try to connect
        self.stream = Some(try!(Connecter::connect(self.host.clone().as_slice(), self.port)));

        // Log the connection
        info!("Connection established to {}[{}]:{}",
              self.host, self.stream.clone().unwrap().peer_name().unwrap().ip, self.port);

        let response = try!(self.stream.clone().unwrap().get_reply());
        let result = try!(response.with_code(vec!(220)));
        self.state = transaction::Connected;
        Ok(result)
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
    pub fn helo<S>(&mut self, my_hostname: String) -> SmtpResult<Response> {
        let res = try!(self.send_command(command::Hello(my_hostname.clone())));
        match res.with_code(vec!(250)) {
            Ok(response) => {
                self.server_info = Some(
                    ServerInfo{
                        name: get_first_word(response.message.clone().unwrap().as_slice()).to_string(),
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
    pub fn ehlo<S>(&mut self, my_hostname: String) -> SmtpResult<Response> {
        let res = try!(self.send_command(command::ExtendedHello(my_hostname.clone())));
        match res.with_code(vec!(250)) {
            Ok(response) => {
                self.server_info = Some(
                    ServerInfo{
                        name: get_first_word(response.message.clone().unwrap().as_slice()).to_string(),
                        esmtp_features: Extension::parse_esmtp_response(
                                            response.message.clone().unwrap().as_slice()
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
                   options: Option<Vec<String>>) -> SmtpResult<Response> {
        let res = try!(self.send_command(
            command::Mail(unquote_email_address(from_address.as_slice()).to_string(), options)
        ));
        match res.with_code(vec!(250)) {
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
                   options: Option<Vec<String>>) -> SmtpResult<Response> {
        let res = try!(self.send_command(
            command::Recipient(unquote_email_address(to_address.as_slice()).to_string(), options)
        ));
        match res.with_code(vec!(250)) {
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
    pub fn data<S>(&mut self) -> SmtpResult<Response> {
        let res = try!(self.send_command(command::Data));
        match res.with_code(vec!(354)) {
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
    pub fn message<S>(&mut self, message_content: &str) -> SmtpResult<Response> {
        let server_info = self.server_info.clone().expect("Bad command sequence");
        // Get maximum message size if defined and compare to the message size
        match server_info.supports_feature(extension::Size(0)) {
            Some(extension::Size(max)) if message_content.len() > max => {
                self.smtp_fail::<S>();
                return Err(FromError::from_error("Message is too big"))
            }
            _ => ()
        };

        // Checks message encoding according to the server's capability
        // TODO : Add an encoding check.
        if ! server_info.supports_feature(extension::EightBitMime).is_some() {
            if ! message_content.clone().is_ascii() {
                self.smtp_fail::<S>();
                return Err(FromError::from_error("Server does not accepts UTF-8 strings"))
            }
        }

        let res = try!(self.send_message(message_content));
        match res.with_code(vec!(250)) {
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
    pub fn quit<S>(&mut self) -> SmtpResult<Response> {
        let res = try!(self.send_command(command::Quit));
        res.with_code(vec!(221))
    }

    /// Sends a RSET command
    pub fn rset<S>(&mut self) -> SmtpResult<Response> {
        let res = try!(self.send_command(command::Reset));
        match res.with_code(vec!(250)) {
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
    pub fn noop<S>(&mut self) -> SmtpResult<Response> {
        let res = try!(self.send_command(command::Noop));
        res.with_code(vec!(250))
    }

    /// Sends a VRFY command
    pub fn vrfy<S, T>(&mut self, to_address: String) -> SmtpResult<Response> {
        let res = try!(self.send_command(command::Verify(to_address, None)));
        res.with_code(vec!(250))
    }

}
