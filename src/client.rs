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
use std::strbuf::StrBuf;
use std::io::{IoResult, Reader, Writer};
use std::io::net::ip::{SocketAddr, Port};
use std::io::net::tcp::TcpStream;

use common::{resolve_host, get_first_word, unquote_email_address};
use smtp::smtp_response::SmtpResponse;
use smtp::esmtp_parameter;
use smtp::esmtp_parameter::EsmtpParameter;
use smtp::smtp_command;
use smtp::smtp_command::SmtpCommand;
use smtp::{SMTP_PORT, CRLF};

/// Information about an SMTP server
#[deriving(Clone)]
struct SmtpServerInfo<T> {
    /// Server name
    name: T,
    /// ESMTP features supported by the server
    esmtp_features: Option<Vec<EsmtpParameter>>
}

impl<T: Show> Show for SmtpServerInfo<T>{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.buf.write(
            format!("{} with {}",
                self.name,
                match self.esmtp_features.clone() {
                    Some(features) => features.to_str(),
                    None => format!("no supported features")
                }
            ).as_bytes()
        )
    }
}

impl<T: Str> SmtpServerInfo<T> {
    /// Parses supported ESMTP features
    ///
    /// TODO: Improve parsing
    fn parse_esmtp_response(message: T) -> Option<Vec<EsmtpParameter>> {
        let mut esmtp_features = Vec::new();
        for line in message.as_slice().split_str(CRLF) {
            match from_str::<SmtpResponse<StrBuf>>(line) {
                Some(SmtpResponse{code: 250, message: message}) => {
                    match from_str::<EsmtpParameter>(message.unwrap().into_owned()) {
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
    fn supports_feature(&self, keyword: EsmtpParameter) -> Result<EsmtpParameter, ()> {
        match self.esmtp_features.clone() {
            Some(esmtp_features) => {
                for feature in esmtp_features.iter() {
                    if keyword.same_keyword_as(*feature) {
                        return Ok(*feature);
                    }
                }
                Err({})
            },
            None => Err({})
        }
    }
}

/// Contains the state of the current transaction
#[deriving(Eq,Clone)]
enum SmtpClientState {
    /// The server is unconnected
    Unconnected,
    /// The connection was successful and the banner was received
    Connected,
    /// An HELO or EHLO was successful
    HeloSent,
    /// A MAIL command was successful send
    MailSent,
    /// At least one RCPT command was sucessful
    RcptSent,
    /// A DATA command was successful
    DataSent
}

macro_rules! check_state_in(
    ($expected_states:expr) => (
        if ! $expected_states.contains(&self.state) {
            fail!("Bad sequence of commands.");
        }
    );
)

macro_rules! check_state_not_in(
    ($expected_states:expr) => (
        if $expected_states.contains(&self.state) {
            fail!("Bad sequence of commands.");
        }
    );
)

macro_rules! smtp_fail_if_err(
    ($response:expr) => (
        match $response {
            Err(response) => {
                self.smtp_fail(response)
            },
            Ok(_) => {}
        }
    );
)

/// Structure that implements the SMTP client
pub struct SmtpClient<T, S> {
    /// TCP stream between client and server
    /// Value is None before connection
    stream: Option<S>,
    /// Host we are connecting to
    host: T,
    /// Port we are connecting on
    port: Port,
    /// Our hostname for HELO/EHLO commands
    my_hostname: T,
    /// Information about the server
    /// Value is None before HELO/EHLO
    server_info: Option<SmtpServerInfo<T>>,
    /// Transaction state, to check the sequence of commands
    state: SmtpClientState
}

impl<S> SmtpClient<StrBuf, S> {
    /// Creates a new SMTP client
    pub fn new(host: StrBuf, port: Option<Port>, my_hostname: Option<StrBuf>) -> SmtpClient<StrBuf, S> {
        SmtpClient{
            stream: None,
            host: host,
            port: port.unwrap_or(SMTP_PORT),
            my_hostname: my_hostname.unwrap_or(StrBuf::from_str("localhost")),
            server_info: None,
            state: Unconnected
        }
    }
}

impl SmtpClient<StrBuf, TcpStream> {
    /// Connects to the configured server
    pub fn connect(&mut self) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        if !self.stream.is_none() {
            fail!("The connection is already established");
        }
        let ip = match resolve_host(self.host.clone().into_owned()) {
            Ok(ip)  => ip,
            Err(..) => fail!("Cannot resolve {:s}", self.host)
        };
        self.stream = match TcpStream::connect(SocketAddr{ip: ip, port: self.port}) {
            Ok(stream) => Some(stream),
            Err(..)    => fail!("Cannot connect to {:s}:{:u}", self.host, self.port)
        };

        // Log the connection
        info!("Connection established to {}[{}]:{}", self.my_hostname.clone(), ip, self.port);

        match self.get_reply() {
            Some(response) => match response.with_code(vec!(220)) {
                                  Ok(response) => {
                                      self.state = Connected;
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
    pub fn send_mail(&mut self, from_address: StrBuf, to_addresses: Vec<StrBuf>, message: StrBuf) {
        let my_hostname = self.my_hostname.clone();

        // Connect
        match self.connect() {
            Ok(_) => {},
            Err(response) => fail!("Cannot connect to {:s}:{:u}. Server says: {}",
                                    self.host,
                                    self.port, response
                             )
        }

        // Extended Hello or Hello
        match self.ehlo(my_hostname.clone()) {
            Err(SmtpResponse{code: 550, message: _}) => {
                smtp_fail_if_err!(self.helo(my_hostname.clone()))
            },
            Err(response) => {
                self.smtp_fail(response)
            }
            _ => {}
        }

        debug!("Server {:s}", self.server_info.clone().unwrap().to_str());

        // Checks message encoding according to the server's capability
        // TODO : Add an encoding check.
        if ! self.server_info.clone().unwrap().supports_feature(esmtp_parameter::EightBitMime).is_ok() {
            if ! message.clone().into_owned().is_ascii() {
                self.smtp_fail("Server does not accepts UTF-8 strings");
            }
        }

        // Mail
        smtp_fail_if_err!(self.mail(from_address.clone(), None));

        // Log the mail command
        info!("from=<{}>, size={}, nrcpt={}", from_address, 42, to_addresses.len());

        // Recipient
        // TODO Return rejected addresses
        // TODO Manage the number of recipients
        for to_address in to_addresses.iter() {
            smtp_fail_if_err!(self.rcpt(to_address.clone(), None));
        }

        // Data
        smtp_fail_if_err!(self.data());

        // Message content
        let sent = self.message(message);

        if sent.clone().is_err() {
            self.smtp_fail(sent.clone().err().unwrap())
        }

        info!("to=<{}>, status=sent ({})", to_addresses.clone().connect(">, to=<"), sent.clone().ok().unwrap());

        // Quit
        smtp_fail_if_err!(self.quit());
    }
}

impl<S: Writer + Reader + Clone> SmtpClient<StrBuf, S> {
    /// Sends an SMTP command
    // TODO : ensure this is an ASCII string
    fn send_command(&mut self, command: SmtpCommand<StrBuf>) -> SmtpResponse<StrBuf> {
        self.send_and_get_response(format!("{}", command))
    }

    /// Sends an email
    fn send_message(&mut self, message: StrBuf) -> SmtpResponse<StrBuf> {
        self.send_and_get_response(format!("{}{:s}.", message, CRLF))
    }

    /// Sends a complete message or a command to the server and get the response
    fn send_and_get_response(&mut self, string: &str) -> SmtpResponse<StrBuf> {
        match (&mut self.stream.clone().unwrap() as &mut Writer)
                .write_str(format!("{:s}{:s}", string, CRLF)) {
            Ok(..)  => debug!("Wrote: {:s}", string),
            Err(..) => fail!("Could not write to stream")
        }

        match self.get_reply() {
            Some(response) => {debug!("Read: {:s}", response.to_str()); response},
            None           => fail!("No answer on {:s}", self.host)
        }
    }

    /// Gets the SMTP response
    fn get_reply(&mut self) -> Option<SmtpResponse<StrBuf>> {
        let response = match self.read_to_str() {
            Ok(string) => string,
            Err(..)    => fail!("No answer")
        };

        from_str::<SmtpResponse<StrBuf>>(response)
    }

    /// Closes the connection and fail with a given messgage
    fn smtp_fail<T: Show>(&mut self, reason: T) {
        if self.is_connected() {
            match self.quit() {
                Ok(..) => {},
                Err(response) => fail!("Failed: {}", response)
            }
        }
        self.close();
        fail!("Failed: {}", reason);
    }

    /// Checks if the server is connected
    pub fn is_connected(&mut self) -> bool {
        self.noop().is_ok()
    }

    /// Closes the TCP stream
    pub fn close(&mut self) {
        // Close the TCP connection
        drop(self.stream.clone().unwrap());
        // Reset client state
        self.stream = None;
        self.state = Unconnected;
        self.server_info = None;
    }

    /// Send a HELO command
    pub fn helo(&mut self, my_hostname: StrBuf) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        check_state_in!(vec!(Connected));

        match self.send_command(smtp_command::Hello(my_hostname.clone())).with_code(vec!(250)) {
            Ok(response) => {
                self.server_info = Some(
                    SmtpServerInfo{
                        name: StrBuf::from_str(get_first_word(response.message.clone().unwrap().into_owned())),
                        esmtp_features: None
                    }
                );
                self.state = HeloSent;
                Ok(response)
            },
            Err(response) => Err(response)
        }
    }

    /// Sends a EHLO command
    pub fn ehlo(&mut self, my_hostname: StrBuf) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        check_state_not_in!(vec!(Unconnected));

        match self.send_command(smtp_command::ExtendedHello(my_hostname.clone())).with_code(vec!(250)) {
            Ok(response) => {
                self.server_info = Some(
                    SmtpServerInfo{
                        name: StrBuf::from_str(get_first_word(response.message.clone().unwrap().to_owned())),
                        esmtp_features: SmtpServerInfo::parse_esmtp_response(response.message.clone().unwrap())
                    }
                );
                self.state = HeloSent;
                Ok(response)
            },
            Err(response) => Err(response)
        }
    }

    /// Sends a MAIL command
    pub fn mail(&mut self, from_address: StrBuf, options: Option<Vec<StrBuf>>) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        check_state_in!(vec!(HeloSent));

        match self.send_command(smtp_command::Mail(StrBuf::from_str(unquote_email_address(from_address.to_owned())), options)).with_code(vec!(250)) {
            Ok(response) => {
                self.state = MailSent;
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends a RCPT command
    pub fn rcpt(&mut self, to_address: StrBuf, options: Option<Vec<StrBuf>>) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        check_state_in!(vec!(MailSent, RcptSent));

        match self.send_command(smtp_command::Recipient(StrBuf::from_str(unquote_email_address(to_address.to_owned())), options)).with_code(vec!(250)) {
            Ok(response) => {
                self.state = RcptSent;
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends a DATA command
    pub fn data(&mut self) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        check_state_in!(vec!(RcptSent));

        match self.send_command(smtp_command::Data).with_code(vec!(354)) {
            Ok(response) => {
                self.state = DataSent;
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends the message content
    pub fn message(&mut self, message_content: StrBuf) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        check_state_in!(vec!(DataSent));

        match self.send_message(message_content).with_code(vec!(250)) {
            Ok(response) => {
                self.state = HeloSent;
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends a QUIT command
    pub fn quit(&mut self) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        check_state_not_in!(vec!(Unconnected));
        match self.send_command(smtp_command::Quit).with_code(vec!(221)) {
            Ok(response) => {
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends a RSET command
    pub fn rset(&mut self) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        check_state_not_in!(vec!(Unconnected));
        match self.send_command(smtp_command::Reset).with_code(vec!(250)) {
            Ok(response) => {
                if vec!(MailSent, RcptSent, DataSent).contains(&self.state) {
                    self.state = HeloSent;
                }
                Ok(response)
            },
            Err(response) => {
                Err(response)
            }
        }
    }

    /// Sends a NOOP commands
    pub fn noop(&mut self) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        check_state_not_in!(vec!(Unconnected));
        self.send_command(smtp_command::Noop).with_code(vec!(250))
    }

    /// Sends a VRFY command
    pub fn vrfy(&mut self, to_address: StrBuf) -> Result<SmtpResponse<StrBuf>, SmtpResponse<StrBuf>> {
        check_state_not_in!(vec!(Unconnected));
        self.send_command(smtp_command::Verify(to_address)).with_code(vec!(250))
    }
}

impl<T, S: Reader + Clone> Reader for SmtpClient<T, S> {
    /// Reads a string from the client socket
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.stream.clone().unwrap().read(buf)
    }

    /// Reads a string from the client socket
    // TODO: Size of response ?.
    fn read_to_str(&mut self) -> IoResult<~str> {
        let mut buf = [0u8, ..1000];

        let response = match self.read(buf) {
            Ok(bytes_read) => from_utf8(buf.slice_to(bytes_read - 1)).unwrap(),
            Err(..)        => fail!("Read error")
        };

        return Ok(response.to_owned());
    }
}

impl<T, S: Writer + Clone> Writer for SmtpClient<T, S> {
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
    use smtp::esmtp_parameter;

    #[test]
    fn test_smtp_server_info_fmt() {
        assert_eq!(format!("{}", SmtpServerInfo{
            name: "name",
            esmtp_features: Some(vec!(esmtp_parameter::EightBitMime))
        }), "name with [8BITMIME]".to_owned());
        assert_eq!(format!("{}", SmtpServerInfo{
            name: "name",
            esmtp_features: Some(vec!(esmtp_parameter::EightBitMime, esmtp_parameter::Size(42)))
        }), "name with [8BITMIME, SIZE=42]".to_owned());
        assert_eq!(format!("{}", SmtpServerInfo{
            name: "name",
            esmtp_features: None
        }), "name with no supported features".to_owned());
    }

    #[test]
    fn test_smtp_server_info_parse_esmtp_response() {
        assert_eq!(SmtpServerInfo::parse_esmtp_response("me\r\n250-8BITMIME\r\n250 SIZE 42"),
            Some(vec!(esmtp_parameter::EightBitMime, esmtp_parameter::Size(42))));
        assert_eq!(SmtpServerInfo::parse_esmtp_response("me\r\n250-8BITMIME\r\n250 UNKNON 42"),
            Some(vec!(esmtp_parameter::EightBitMime)));
        assert_eq!(SmtpServerInfo::parse_esmtp_response("me\r\n250-9BITMIME\r\n250 SIZE a"),
            None);
        assert_eq!(SmtpServerInfo::parse_esmtp_response("me\r\n250-SIZE 42\r\n250 SIZE 43"),
            Some(vec!(esmtp_parameter::Size(42), esmtp_parameter::Size(43))));
        assert_eq!(SmtpServerInfo::parse_esmtp_response(""),
            None);
    }

    #[test]
    fn test_smtp_server_info_supports_feature() {
        assert_eq!(SmtpServerInfo{
            name: "name",
            esmtp_features: Some(vec!(esmtp_parameter::EightBitMime))
        }.supports_feature(esmtp_parameter::EightBitMime), Ok(esmtp_parameter::EightBitMime));
        assert_eq!(SmtpServerInfo{
            name: "name",
            esmtp_features: Some(vec!(esmtp_parameter::Size(42), esmtp_parameter::EightBitMime))
        }.supports_feature(esmtp_parameter::EightBitMime), Ok(esmtp_parameter::EightBitMime));
        assert_eq!(SmtpServerInfo{
            name: "name",
            esmtp_features: Some(vec!(esmtp_parameter::Size(42), esmtp_parameter::EightBitMime))
        }.supports_feature(esmtp_parameter::Size(0)), Ok(esmtp_parameter::Size(42)));
        assert!(SmtpServerInfo{
            name: "name",
            esmtp_features: Some(vec!(esmtp_parameter::EightBitMime))
        }.supports_feature(esmtp_parameter::Size(42)).is_err());
    }
}
