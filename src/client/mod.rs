// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP client

use std::fmt;
use std::fmt::{Show, Formatter};
use std::str::from_utf8;
use std::result::Result;
use std::string::String;
use std::io::{IoResult, Reader, Writer};
use std::io::net::ip::Port;

use common::{get_first_word, unquote_email_address};
use response::SmtpResponse;
use extension;
use extension::SmtpExtension;
use command;
use command::SmtpCommand;
use common::{SMTP_PORT, CRLF};
use transaction;
use transaction::TransactionState;
use client::connecter::Connecter;

mod connecter;

/// Information about an SMTP server
#[deriving(Clone)]
struct SmtpServerInfo {
    /// Server name
    name: String,
    /// ESMTP features supported by the server
    esmtp_features: Option<Vec<SmtpExtension>>
}

impl Show for SmtpServerInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write(
            format!("{} with {}",
                self.name,
                match self.esmtp_features.clone() {
                    Some(features) => features.to_string(),
                    None => format!("no supported features")
                }
            ).as_bytes()
        )
    }
}

impl SmtpServerInfo {
    /// Parses supported ESMTP features
    ///
    /// TODO: Improve parsing
    fn parse_esmtp_response(message: String) -> Option<Vec<SmtpExtension>> {
        let mut esmtp_features = Vec::new();
        for line in message.as_slice().split_str(CRLF) {
            match from_str::<SmtpResponse>(line) {
                Some(SmtpResponse{code: 250, message: message}) => {
                    match from_str::<SmtpExtension>(message.unwrap().as_slice()) {
                        Some(keyword) => esmtp_features.push(keyword),
                        None          => ()
                    }
                },
                _ => ()
            }
        }
        match esmtp_features.len() {
            0 => None,
            _ => Some(esmtp_features)
        }
    }

    /// Checks if the server supports an ESMTP feature
    fn supports_feature(&self, keyword: SmtpExtension) -> Result<SmtpExtension, ()> {
        match self.esmtp_features.clone() {
            Some(esmtp_features) => {
                for feature in esmtp_features.iter() {
                    if keyword.same_extension_as(*feature) {
                        return Ok(*feature);
                    }
                }
                Err({})
            },
            None => Err({})
        }
    }
}

/// Structure that implements the SMTP client
pub struct SmtpClient<S> {
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
    server_info: Option<SmtpServerInfo>,
    /// Transaction state, to check the sequence of commands
    state: TransactionState
}

impl<S> SmtpClient<S> {
    /// Creates a new SMTP client
    pub fn new(host: String, port: Option<Port>, my_hostname: Option<String>) -> SmtpClient<S> {
        SmtpClient{
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
impl<S: Connecter + Reader + Writer + Clone> SmtpClient<S> {

    /// TODO
    fn smtp_fail_if_err<S>(&mut self, response: Result<SmtpResponse, SmtpResponse>) {
        match response {
            Err(response) => {
                self.smtp_fail::<S, SmtpResponse>(response)
            },
            Ok(_) => {}
        }
    }

    /// Connects to the configured server
    pub fn connect(&mut self) -> Result<SmtpResponse, SmtpResponse> {
        // connect should not be called when the client is already connected
        if !self.stream.is_none() {
            fail!("The connection is already established");
        }

        // Try to connect
        self.stream = match Connecter::connect(self.host.clone().as_slice(), self.port) {
            Ok(stream) => Some(stream),
            Err(..) => fail!("Cannot connect to the server")
        };

        // Log the connection
        info!("Connection established to {}[{}]:{}", self.host, self.stream.clone().unwrap().peer_name().unwrap().ip, self.port);

        match self.get_reply() {
            Some(response) => match response.with_code(vec!(220)) {
                                  Ok(response)  => {
                                      self.state = transaction::Connected;
                                      Ok(response)
                                  },
                                  Err(response) => {
                                      Err(response)
                                  }
                              },
            None           => fail!("No banner on {}", self.host)
        }
    }

    /// Sends an email
    pub fn send_mail<S>(&mut self, from_address: String, to_addresses: Vec<String>, message: String) {
        let my_hostname = self.my_hostname.clone();
        let mut smtp_result: Result<SmtpResponse, SmtpResponse>;

        match self.connect() {
            Ok(_) => {},
            Err(response) => fail!("Cannot connect to {:s}:{:u}. Server says: {}",
                                    self.host,
                                    self.port, response
                             )
        }

        // Extended Hello or Hello
        match self.ehlo::<S>(my_hostname.clone().to_string()) {
            Err(SmtpResponse{code: 550, message: _}) => {
                smtp_result = self.helo::<S>(my_hostname.clone());
                self.smtp_fail_if_err::<S>(smtp_result);
            },
            Err(response) => {
                self.smtp_fail::<S, SmtpResponse>(response)
            }
            _ => {}
        }

        debug!("Server {}", self.server_info.clone().unwrap());

        // Checks message encoding according to the server's capability
        // TODO : Add an encoding check.
        if ! self.server_info.clone().unwrap().supports_feature(extension::EightBitMime).is_ok() {
            if ! message.clone().to_string().is_ascii() {
                self.smtp_fail::<S, &str>("Server does not accepts UTF-8 strings");
            }
        }

        // Mail
        smtp_result = self.mail::<S>(from_address.clone(), None);
        self.smtp_fail_if_err::<S>(smtp_result);

        // Log the mail command
        info!("from=<{}>, size={}, nrcpt={}", from_address, 42u, to_addresses.len());

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
            self.smtp_fail::<S, SmtpResponse>(sent.clone().err().unwrap())
        }

        info!("to=<{}>, status=sent ({})", to_addresses.clone().connect(">, to=<"), sent.clone().ok().unwrap());

        // Quit
        smtp_result = self.quit::<S>();
        self.smtp_fail_if_err::<S>(smtp_result);
    }

    /// Sends an SMTP command
    // TODO : ensure this is an ASCII string
    fn send_command(&mut self, command: SmtpCommand) -> SmtpResponse {
        if !self.state.is_command_possible(command.clone()) {
            fail!("Bad command sequence");
        }
        self.send_and_get_response(format!("{}", command).as_slice())
    }

    /// Sends an email
    fn send_message(&mut self, message: String) -> SmtpResponse {
        self.send_and_get_response(format!("{}{:s}.", message, CRLF).as_slice())
    }

    /// Sends a complete message or a command to the server and get the response
    fn send_and_get_response(&mut self, string: &str) -> SmtpResponse {
        match (&mut self.stream.clone().unwrap() as &mut Writer)
                .write_str(format!("{:s}{:s}", string, CRLF).as_slice()) { // TODO improve this
            Ok(..)  => debug!("Wrote: {:s}", string),
            Err(..) => fail!("Could not write to stream")
        }

        match self.get_reply() {
            Some(response) => {debug!("Read: {}", response); response},
            None           => fail!("No answer on {:s}", self.host)
        }
    }

    /// Gets the SMTP response
    fn get_reply(&mut self) -> Option<SmtpResponse> {
        let response = match self.read_to_string() {
            Ok(string) => string,
            Err(..)    => fail!("No answer")
        };
        from_str::<SmtpResponse>(response.as_slice())
    }

    /// Closes the connection and fail with a given messgage
    fn smtp_fail<S, T: Show>(&mut self, reason: T) {
        let is_connected = self.is_connected::<S>();
        if is_connected {
            match self.quit::<S>() {
                Ok(..) => {},
                Err(response) => fail!("Failed: {}", response)
            }
        }
        self.close();
        fail!("Failed: {}", reason);
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
    pub fn helo<S>(&mut self, my_hostname: String) -> Result<SmtpResponse, SmtpResponse> {
        match self.send_command(command::Hello(my_hostname.clone())).with_code(vec!(250)) {
            Ok(response) => {
                self.server_info = Some(
                    SmtpServerInfo{
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
    pub fn ehlo<S>(&mut self, my_hostname: String) -> Result<SmtpResponse, SmtpResponse> {
        match self.send_command(command::ExtendedHello(my_hostname.clone())).with_code(vec!(250)) {
            Ok(response) => {
                self.server_info = Some(
                    SmtpServerInfo{
                        name: get_first_word(response.message.clone().unwrap()),
                        esmtp_features: SmtpServerInfo::parse_esmtp_response(response.message.clone().unwrap())
                    }
                );
                self.state = transaction::HelloSent;
                Ok(response)
            },
            Err(response) => Err(response)
        }
    }

    /// Sends a MAIL command
    pub fn mail<S>(&mut self, from_address: String, options: Option<Vec<String>>) -> Result<SmtpResponse, SmtpResponse> {
        match self.send_command(command::Mail(unquote_email_address(from_address.to_string()), options)).with_code(vec!(250)) {
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
    pub fn rcpt<S>(&mut self, to_address: String, options: Option<Vec<String>>) -> Result<SmtpResponse, SmtpResponse> {
        match self.send_command(command::Recipient(unquote_email_address(to_address.to_string()), options)).with_code(vec!(250)) {
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
    pub fn data<S>(&mut self) -> Result<SmtpResponse, SmtpResponse> {
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
    pub fn message<S>(&mut self, message_content: String) -> Result<SmtpResponse, SmtpResponse> {
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
    pub fn quit<S>(&mut self) -> Result<SmtpResponse, SmtpResponse> {
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
    pub fn rset<S>(&mut self) -> Result<SmtpResponse, SmtpResponse> {
        match self.send_command(command::Reset).with_code(vec!(250)) {
            Ok(response) => {
                if vec!(transaction::MailSent, transaction::RecipientSent, transaction::DataSent).contains(&self.state) {
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
    pub fn noop<S>(&mut self) -> Result<SmtpResponse, SmtpResponse> {
        self.send_command(command::Noop).with_code(vec!(250))
    }

    /// Sends a VRFY command
    pub fn vrfy<S, T>(&mut self, to_address: String) -> Result<SmtpResponse, SmtpResponse> {
        self.send_command(command::Verify(to_address, None)).with_code(vec!(250))
    }
}

impl<S: Reader + Clone> Reader for SmtpClient<S> {
    /// Reads a string from the client socket
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.stream.clone().unwrap().read(buf)
    }

    /// Reads a string from the client socket
    // TODO: Size of response ?.
    fn read_to_string(&mut self) -> IoResult<String> {
        let mut buf = [0u8, ..1034];

        let response = match self.read(buf) {
            Ok(bytes_read) => from_utf8(buf.slice_to(bytes_read - 1)).unwrap(),
            Err(..)        => fail!("Read error")
        };

        return Ok(response.to_string());
    }
}

impl<S: Writer + Clone> Writer for SmtpClient<S> {
    /// Sends a string on the client socket
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.stream.clone().unwrap().write(buf)
    }

    /// Sends a string on the client socket
    fn write_str(&mut self, string: &str) -> IoResult<()> {
        self.stream.clone().unwrap().write_str(string)
    }
}

#[cfg(test)]
mod test {
    use super::SmtpServerInfo;
    use extension;

    #[test]
    fn test_smtp_server_info_fmt() {
        assert_eq!(format!("{}", SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::EightBitMime))
        }), "name with [8BITMIME]".to_string());
        assert_eq!(format!("{}", SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::EightBitMime, extension::Size(42)))
        }), "name with [8BITMIME, SIZE=42]".to_string());
        assert_eq!(format!("{}", SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: None
        }), "name with no supported features".to_string());
    }

    #[test]
    fn test_smtp_server_info_parse_esmtp_response() {
        assert_eq!(SmtpServerInfo::parse_esmtp_response(String::from_str("me\r\n250-8BITMIME\r\n250 SIZE 42")),
            Some(vec!(extension::EightBitMime, extension::Size(42))));
        assert_eq!(SmtpServerInfo::parse_esmtp_response(String::from_str("me\r\n250-8BITMIME\r\n250 UNKNON 42")),
            Some(vec!(extension::EightBitMime)));
        assert_eq!(SmtpServerInfo::parse_esmtp_response(String::from_str("me\r\n250-9BITMIME\r\n250 SIZE a")),
            None);
        assert_eq!(SmtpServerInfo::parse_esmtp_response(String::from_str("me\r\n250-SIZE 42\r\n250 SIZE 43")),
            Some(vec!(extension::Size(42), extension::Size(43))));
        assert_eq!(SmtpServerInfo::parse_esmtp_response(String::from_str("")),
            None);
    }

    #[test]
    fn test_smtp_server_info_supports_feature() {
        assert_eq!(SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::EightBitMime))
        }.supports_feature(extension::EightBitMime), Ok(extension::EightBitMime));
        assert_eq!(SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::Size(42), extension::EightBitMime))
        }.supports_feature(extension::EightBitMime), Ok(extension::EightBitMime));
        assert_eq!(SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::Size(42), extension::EightBitMime))
        }.supports_feature(extension::Size(0)), Ok(extension::Size(42)));
        assert!(SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::EightBitMime))
        }.supports_feature(extension::Size(42)).is_err());
    }
}
