/*!

Simple SMTP client, without ESMTP and SSL/TLS support for now.

# Usage

```
let mut email_client: SmtpClient = SmtpClient::new("localhost", None, "myhost.example.org");
email_client.send_mail("user@example.org", [&"user@localhost"], "Message content.");
```

# TODO

 Support ESMTP : Parse server answer, and manage mail and rcpt options.

* Client options: `mail_options` and `rcpt_options` lists

* Server options: helo/ehlo, parse and store ehlo response

Manage errors

Support SSL/TLS

*/

use std::fmt;
use std::str::from_utf8;
use std::io::{IoResult, IoError};
use std::io::net::ip::{SocketAddr, Port};
use std::io::net::tcp::TcpStream;
use std::io::net::addrinfo::get_host_addresses;
use common::{SMTP_PORT, CRLF};
use commands;

/// Contains an SMTP reply, with separed code and message
pub struct SmtpResponse {
    /// Server respinse code code
    code: uint,
    /// Server response string
    message: ~str
}

impl ToStr for SmtpResponse {
    /// Get the server reply
    fn to_str(&self) -> ~str {
        return format!("{} {}", self.code.to_str(), self.message);
    }
}

impl fmt::Show for SmtpResponse {
    /// Format SMTP response display
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), IoError> {
        f.buf.write(self.to_str().as_bytes())
    }
}

impl SmtpResponse {
    /// Check the repsonse code and fail if there is an error
    fn check_response(&self, expected_codes: &[uint]) {
        for &code in expected_codes.iter() {
            if code == self.code {
                return;
            }
        }
        fail!("Failed with {}", self.to_str());
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
    my_hostname: ~str
}

impl<S: Reader + Writer + Clone> SmtpClient<S> {

    /// Send an SMTP command
    pub fn send_command(&mut self, command: commands::Command, option: Option<~str>) -> SmtpResponse {
        self.send_and_get_response(commands::SmtpCommand::new(command, option).get_formatted_command())
    }

    /// Send an email
    pub fn send_message(&mut self, message: ~str) -> SmtpResponse {
        self.send_and_get_response(format!("{:s}{:s}.", message, CRLF))
    }

    /// Send a complete message or a command to the server and get the response
    fn send_and_get_response(&mut self, string: ~str) -> SmtpResponse {
        match (&mut self.stream.clone().unwrap() as &mut Writer).write_str(format!("{:s}{:s}", string, CRLF)) {
            Err(..) => fail!("Could not write to stream"),
            Ok(..) => debug!("Write success")
        }

        match self.get_reply() {
            None => fail!("No answer on {}", self.host),
            Some(response) => response
        }
    }

    /// Get the SMTP response
    fn get_reply(&mut self) -> Option<SmtpResponse> {
        let response = match self.stream.clone().unwrap().read_to_str() {
            Err(..) => fail!("No answer"),
            Ok(string) => string
        };

        if response.len() > 4 {
            Some(SmtpResponse {
                    code: from_str(response.slice_to(3)).unwrap(),
                    message: response.slice_from(4).to_owned()
                 })
        } else {
            None
        }
    }

    /// Create a new SMTP client
    pub fn new(host: &str, port: Option<Port>, my_hostname: Option<&str>) -> SmtpClient<S> {
        SmtpClient{
            stream: None,
            host: host.to_owned(),
            port: port.unwrap_or(SMTP_PORT),
            my_hostname: my_hostname.unwrap_or("localhost").to_owned(),
        }
    }
}

impl SmtpClient<TcpStream> {

    /// Send an email
    pub fn send_mail(&mut self, from_addr: &str, to_addrs: &[&str], message: &str) {
        let my_hostname = self.my_hostname.clone();
        self.connect().check_response([220]);
        self.send_command(commands::Hello, Some(my_hostname)).check_response([250]);
        self.send_command(commands::Mail, Some(from_addr.to_owned())).check_response([250]);
        for &to_addr in to_addrs.iter() {
            self.send_command(commands::Recipient, Some(to_addr.to_owned())).check_response([250]);
        }
        self.send_command(commands::Data, None).check_response([354]);
        self.send_message(message.to_owned()).check_response([250]);
        self.send_command(commands::Quit, None).check_response([221]);
    }

    /// Connect to the configured server
    pub fn connect(&mut self) -> SmtpResponse {

        if !self.stream.is_none() {
            fail!("The connection is already established");
        }

        let ip = match get_host_addresses(self.host.clone()) {
            Ok(ip_vector) => ip_vector[0],
            Err(..)    => fail!("Cannot resolve {}", self.host)
        };

        self.stream = match TcpStream::connect(SocketAddr{ip: ip, port: self.port}) {
            Err(..) => fail!("Cannot connect to {}:{}", self.host, self.port),
            Ok(stream) => Some(stream)
        };

        match self.get_reply() {
            None => fail!("No banner on {}", self.host),
            Some(response) => response
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
            Err(..) => fail!("Read error"),
            Ok(bytes_read) => from_utf8(buf.slice_to(bytes_read - 1)).unwrap()
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
