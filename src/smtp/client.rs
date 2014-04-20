/*!

Simple SMTP client.

# Usage

```
let mut email_client: SmtpClient<TcpStream> = SmtpClient::new("localhost", None, None);
email_client.send_mail("user@example.org", [&"user@example.com"], "Example email");
```

*/

use std::fmt;
use std::from_str;
use std::str::from_utf8;
use std::result::Result;
use std::io::{IoResult, IoError};
use std::io::net::ip::{SocketAddr, Port};
use std::io::net::tcp::TcpStream;
use std::io::net::addrinfo::get_host_addresses;
use common::{SMTP_PORT, CRLF, get_first_word};
use commands;
use commands::{Command, SmtpCommand, EhloKeyword};

// Define smtp_fail! and smtp_success!

/// Contains an SMTP reply, with separed code and message
#[deriving(Eq,Clone)]
pub struct SmtpResponse {
    /// Server respinse code code
    code: uint,
    /// Server response string
    message: ~str
}

impl fmt::Show for SmtpResponse {
    /// Format SMTP response display
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), IoError> {
        f.buf.write(
            format!("{} {}", self.code.to_str(), self.message).as_bytes()
        )
    }
}

impl from_str::FromStr for SmtpResponse {
    /// Parse an SMTP response line
    fn from_str(s: &str) -> Option<SmtpResponse> {
        if s.len() < 5 {
            None
        } else {
            if [" ", "-"].contains(&s.slice(3,4)) {
                Some(SmtpResponse{
                    code: from_str(s.slice_to(3)).unwrap(),
                    message: s.slice_from(4).to_owned()
                })
            } else {
                None
            }
        }
    }
}

impl SmtpResponse {
    /// Check the response code
    fn with_code(&self, expected_codes: &[uint]) -> Result<SmtpResponse,SmtpResponse> {
        let response = SmtpResponse{code: self.code, message: self.message.clone()};
        for &code in expected_codes.iter() {
            if code == self.code {
                return Ok(response);
            }
        }
        return Err(response);
    }
}

/// Information about an SMTP server
#[deriving(Eq,Clone)]
pub struct SmtpServerInfo {
    /// Server name
    name: ~str,
    /// ESMTP features supported by the server
    esmtp_features: Option<~[EhloKeyword]>
}

impl SmtpServerInfo {
    /// Parse supported ESMTP features
    fn parse_esmtp_response(message: &str) -> Option<~[EhloKeyword]> {
        let mut esmtp_features: ~[EhloKeyword] = ~[];
        for line in message.split_str(CRLF) {
            match from_str::<SmtpResponse>(line) {
                Some(SmtpResponse{code: 250, message: message}) => {
                    match from_str::<EhloKeyword>(message) {
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
    fn supports_feature(&self, keyword: EhloKeyword) -> bool {
        match self.esmtp_features.clone() {
            Some(esmtp_features) => {
                esmtp_features.contains(&keyword)
            },
            None => false
        }
    }
}

impl fmt::Show for SmtpServerInfo {
    /// Format SMTP server information display
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), IoError> {
        f.buf.write(
            format!("{:s} with {}", self.name, self.esmtp_features).as_bytes()
        )
    }
}

/// Structure that implements a simple SMTP client
pub struct SmtpClient<S> {
    /// TCP stream between client and server
    stream: Option<S>,
    /// Host we are connecting to
    host: ~str,
    /// Port we are connecting on
    port: Port,
    /// Our hostname for HELO/EHLO commands
    my_hostname: ~str,
    /// Information about the server
    server_info: Option<SmtpServerInfo>
}

impl<S> SmtpClient<S> {
    /// Create a new SMTP client
    pub fn new(host: &str, port: Option<Port>, my_hostname: Option<&str>) -> SmtpClient<S> {
        SmtpClient{
            stream: None,
            host: host.to_owned(),
            port: port.unwrap_or(SMTP_PORT),
            my_hostname: my_hostname.unwrap_or("localhost").to_owned(),
            server_info: None
        }
    }
}

impl SmtpClient<TcpStream> {
    /// Send an SMTP command
    pub fn send_command(&mut self, command: Command, option: Option<~str>) -> SmtpResponse {
        self.send_and_get_response(SmtpCommand::new(command, option).to_str())
    }

    /// Send an email
    pub fn send_message(&mut self, message: ~str) -> SmtpResponse {
        self.send_and_get_response(format!("{:s}{:s}.", message, CRLF))
    }

    /// Send a complete message or a command to the server and get the response
    fn send_and_get_response(&mut self, string: ~str) -> SmtpResponse {
        match (&mut self.stream.clone().unwrap() as &mut Writer)
                .write_str(format!("{:s}{:s}", string, CRLF)) {
            Ok(..)  => debug!("Write success"),
            Err(..) => fail!("Could not write to stream")
        }

        match self.get_reply() {
            Some(response) => response,
            None           => fail!("No answer on {}", self.host)
        }
    }

    /// Get the SMTP response
    fn get_reply(&mut self) -> Option<SmtpResponse> {
        let response = match self.read_to_str() {
            Ok(string) => string,
            Err(..)    => fail!("No answer")
        };

        from_str::<SmtpResponse>(response)
    }

    /// Connect to the configured server
    pub fn connect(&mut self) -> SmtpResponse {
        if !self.stream.is_none() {
            fail!("The connection is already established");
        }
        let ip = match get_host_addresses(self.host.clone()) {
            Ok(ip_vector) => ip_vector[0],
            Err(..)       => fail!("Cannot resolve {}", self.host)
        };
        self.stream = match TcpStream::connect(SocketAddr{ip: ip, port: self.port}) {
            Ok(stream) => Some(stream),
            Err(..)    => fail!("Cannot connect to {}:{}", self.host, self.port)
        };
        match self.get_reply() {
            Some(response) => response,
            None           => fail!("No banner on {}", self.host)
        }
    }

    /// Print an SMTP response as info
    fn smtp_success(&mut self, response: SmtpResponse) {
        info!("{:u} {:s}", response.code, response.message);
    }

    /// Send a QUIT command and end the program
    fn smtp_fail(&mut self, command: ~str, reason: &str) {
        self.send_command(commands::Quit, None);
        fail!("{} failed: {:s}", command, reason);
    }

    /// Send an email
    pub fn send_mail(&mut self, from_addr: &str, to_addrs: &[&str], message: &str) {
        let my_hostname = self.my_hostname.clone();

        // Connect
        match self.connect().with_code([220]) {
            Ok(response)  => self.smtp_success(response),
            Err(response) => self.smtp_fail(~"CONNECT", response.to_str())
        }

        // Extended Hello or Hello
        match self.send_command(commands::Ehlo, Some(my_hostname.clone())).with_code([250, 500]) {
            Ok(SmtpResponse{code: 250, message: message}) => {
                self.server_info = Some(
                    SmtpServerInfo{
                        name: get_first_word(message.clone()), 
                        esmtp_features: SmtpServerInfo::parse_esmtp_response(message.clone())
                    }
                );
                self.smtp_success(SmtpResponse{code: 250u, message: message});
            },
            Ok(..) => {
                match self.send_command(commands::Helo, Some(my_hostname.clone())).with_code([250]) {
                    Ok(response) => {
                        self.server_info = Some(
                            SmtpServerInfo{
                                name: get_first_word(response.message.clone()), 
                                esmtp_features: None
                            }
                        );
                        self.smtp_success(response);
                    },
                    Err(response) => self.smtp_fail(~"HELO", response.to_str())
                }
            },
            Err(response) => self.smtp_fail(~"EHLO", response.to_str())
        }

        debug!("SMTP server : {:s}", self.server_info.clone().unwrap().to_str())

        // Check message encoding according to the server's capability
        if ! self.server_info.clone().unwrap().supports_feature(commands::EightBitMime) {
            if ! message.is_ascii() {
                self.smtp_fail(~"DATA", "Server does not accepts UTF-8 strings")
            }
        }

        // Mail
        match self.send_command(commands::Mail, Some(from_addr.to_owned())).with_code([250]) {
            Ok(response)  => self.smtp_success(response),
            Err(response) => self.smtp_fail(~"MAIL", response.to_str())
        }

        // Recipient
        for &to_addr in to_addrs.iter() {
            match self.send_command(commands::Rcpt, Some(to_addr.to_owned())).with_code([250]) {
                Ok(response)  => self.smtp_success(response),
                Err(response) => self.smtp_fail(~"RCPT", response.to_str())
            }
        }

        // Data
        match self.send_command(commands::Data, None).with_code([354]) {
                Ok(response)  => self.smtp_success(response),
                Err(response) => self.smtp_fail(~"DATA", response.to_str())
        }

        // Message content
        match self.send_message(message.to_owned()).with_code([250]) {
                Ok(response)  => self.smtp_success(response),
                Err(response) => self.smtp_fail(~"MESSAGE", response.to_str())
        }

        // Quit
        match self.send_command(commands::Quit, None).with_code([221]) {
                Ok(response)  => self.smtp_success(response),
                Err(response) => self.smtp_fail(~"DATA", response.to_str())
        }
    }
}

impl Reader for SmtpClient<TcpStream> {
    /// Read a string from the client socket
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.stream.clone().unwrap().read(buf)
    }

    /// Read a string from the client socket
    fn read_to_str(&mut self) -> IoResult<~str> {
        let mut buf = [0u8, ..1000];

        let response = match self.read(buf) {
            Ok(bytes_read) => from_utf8(buf.slice_to(bytes_read - 1)).unwrap(),
            Err(..)        => fail!("Read error")
        };
        debug!("Read: {:s}", response);

        return Ok(response.to_owned());
    }
}

impl Writer for SmtpClient<TcpStream> {
    /// Send a string on the client socket
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.stream.clone().unwrap().write(buf)
    }

    /// Send a string on the client socket
    fn write_str(&mut self, string: &str) -> IoResult<()> {
        debug!("Wrote: {:s}", string);
        self.stream.clone().unwrap().write_str(string)
    }
}
