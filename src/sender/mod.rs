// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Sends an email using the client

use std::string::String;
use std::net::{SocketAddr, ToSocketAddrs};

use uuid::Uuid;

use SMTP_PORT;
use extension::Extension;
use error::{SmtpResult, SmtpError};
use sendable_email::SendableEmail;
use sender::server_info::ServerInfo;
use client::Client;
use client::net::SmtpStream;

mod server_info;

/// Contains client configuration
pub struct SenderBuilder {
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

/// Builder for the SMTP Sender
impl SenderBuilder {
    /// Creates a new local SMTP client
    pub fn new<A: ToSocketAddrs>(addr: A) -> SenderBuilder {
        SenderBuilder {
            server_addr: addr.to_socket_addrs().ok().expect("could not parse server address").next().unwrap(),
            credentials: None,
            connection_reuse_count_limit: 100,
            enable_connection_reuse: false,
            hello_name: "localhost".to_string(),
        }
    }

    /// Creates a new local SMTP client to port 25
    pub fn localhost() -> SenderBuilder {
        SenderBuilder::new(("localhost", SMTP_PORT))
    }

    /// Set the name used during HELO or EHLO
    pub fn hello_name(mut self, name: &str) -> SenderBuilder {
        self.hello_name = name.to_string();
        self
    }

    /// Enable connection reuse
    pub fn enable_connection_reuse(mut self, enable: bool) -> SenderBuilder {
        self.enable_connection_reuse = enable;
        self
    }

    /// Set the maximum number of emails sent using one connection
    pub fn connection_reuse_count_limit(mut self, limit: u16) -> SenderBuilder {
        self.connection_reuse_count_limit = limit;
        self
    }

    /// Set the client credentials
    pub fn credentials(mut self, username: &str, password: &str) -> SenderBuilder {
        self.credentials = Some((username.to_string(), password.to_string()));
        self
    }

    /// Build the SMTP client
    ///
    /// It does not connects to the server, but only creates the `Sender`
    pub fn build(self) -> Sender {
        Sender::new(self)
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

/// Structure that implements the high level SMTP client
pub struct Sender {
    /// Information about the server
    /// Value is None before HELO/EHLO
    server_info: Option<ServerInfo>,
    /// Sender variable states
    state: State,
    /// Information about the client
    client_info: SenderBuilder,
    /// Low level client
    client: Client<SmtpStream>,
}

macro_rules! try_smtp (
    ($err: expr, $client: ident) => ({
        match $err {
            Ok(val) => val,
            Err(err) => {
                if !$client.state.panic {
                    $client.state.panic = true;
                    $client.reset();
                }
                return Err(err)
            },
        }
    })
);

impl Sender {
    /// Creates a new SMTP client
    ///
    /// It does not connects to the server, but only creates the `Sender`
    pub fn new(builder: SenderBuilder) -> Sender {
        let client: Client<SmtpStream> = Client::new(builder.server_addr);
        Sender{
            client: client,
            server_info: None,
            client_info: builder,
            state: State {
                panic: false,
                connection_reuse_count: 0,
            },
        }
    }

    /// Reset the client state
    fn reset(&mut self) {
        // Close the SMTP transaction if needed
        self.close();

        // Reset the client state
        self.server_info = None;
        self.state.panic = false;
        self.state.connection_reuse_count = 0;
    }

    /// Closes the inner connection
    pub fn close(&mut self) {
        self.client.close();
    }

    /// Sends an email
    pub fn send<T: SendableEmail>(&mut self, mut email: T) -> SmtpResult {
        // Check if the connection is still available
        if self.state.connection_reuse_count > 0 {
            if !self.client.is_connected() {
                self.reset();
            }
        }

        // If there is a usable connection, test if the server answers and hello has been sent
        if self.state.connection_reuse_count == 0 {
            try!(self.client.connect());

            // Log the connection
            info!("connection established to {}", self.client_info.server_addr);

            // Extended Hello or Hello if needed
            match self.client.ehlo(&self.client_info.hello_name) {
                Ok(response) => {self.server_info = Some(
                    ServerInfo{
                        name: response.first_word().expect("Server announced no hostname"),
                        esmtp_features: Extension::parse_esmtp_response(&response),
                    });
                },
                Err(error) => match error {
                    SmtpError::PermanentError(ref response) if response.has_code(550) => {
                        match self.client.helo(&self.client_info.hello_name) {
                            Ok(response) => {self.server_info = Some(
                                ServerInfo{
                                    name: response.first_word().expect("Server announced no hostname"),
                                    esmtp_features: vec!(),
                                });
                            },
                            Err(error) => try_smtp!(Err(error), self)
                        }

                    },
                    _ => {
                        try_smtp!(Err(error), self)
                    },
                },
            }

            // Print server information
            debug!("server {}", self.server_info.as_ref().unwrap());
        }

        // TODO: Use PLAIN AUTH in encrypted connections, CRAM-MD5 otherwise
        if self.client_info.credentials.is_some() && self.state.connection_reuse_count == 0 {

            let (username, password) = self.client_info.credentials.clone().unwrap();

            if self.server_info.as_ref().unwrap().supports_feature(Extension::CramMd5Authentication) {
                let result = self.client.auth_cram_md5(&username, &password);
                try_smtp!(result, self);
            } else if self.server_info.as_ref().unwrap().supports_feature(Extension::PlainAuthentication) {
                let result = self.client.auth_plain(&username, &password);
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
        let mail_options = match self.server_info.as_ref().unwrap().supports_feature(Extension::EightBitMime) {
            true => Some("BODY=8BITMIME"),
            false => None,
        };

        try_smtp!(self.client.mail(&from_address, mail_options), self);

        // Log the mail command
        info!("{}: from=<{}>", current_message, from_address);

        // Recipient
        for to_address in to_addresses.iter() {
            try_smtp!(self.client.rcpt(&to_address), self);
            // Log the rcpt command
            info!("{}: to=<{}>", current_message, to_address);
        }

        // Data
        try_smtp!(self.client.data(), self);

        // Message content
        let result = self.client.message(&message);

        if result.is_ok() {
            // Increment the connection reuse counter
            self.state.connection_reuse_count = self.state.connection_reuse_count + 1;

            // Log the message
            info!("{}: conn_use={}, size={}, status=sent ({})", current_message,
                self.state.connection_reuse_count, message.len(),
                result.as_ref().ok().unwrap().message().iter().next().unwrap_or(&"no response".to_string())
            );
        }

        // Test if we can reuse the existing connection
        if (!self.client_info.enable_connection_reuse) ||
            (self.state.connection_reuse_count >= self.client_info.connection_reuse_count_limit) {
            self.reset();
        }

        result
    }
}
