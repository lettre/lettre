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

use std::str::from_utf8;
use std::io::net::ip::{SocketAddr, Port};
use std::io::net::tcp::TcpStream;
use std::io::net::addrinfo::get_host_addresses;
use common::{SMTP_PORT, CRLF};
use commands::SmtpCommand;

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

/// Structure that implements a simple SMTP client
pub struct SmtpClient {
    /// TCP socket between client and server
    socket: Option<TcpStream>,
    /// Reading buffer
    buf: [u8, ..1000],
    /// Host we are connecting to
    host: ~str,
    /// Port we are connecting on
    port: Port,
    /// Our hostname for HELO/EHLO commands
    my_hostname: ~str
}

impl SmtpClient {

    /// Connect to the configured server
    pub fn connect(&mut self) -> SmtpResponse {
        let ip = match get_host_addresses(self.host.clone()) {
            Ok(ip_vector) => ip_vector[0],
            Err(error)    => fail!("Cannot resolve {}", self.host)
        };


        self.socket = match TcpStream::connect(SocketAddr{ip: ip, port: self.port}) {
            Err(error) => fail!("Cannot connect to {}:{}", self.host, self.port),
            Ok(socket) => Some(socket)
        };

        match self.get_reply() {
            None => fail!("No banner on {}", self.host),
            Some(response) => response
        }
    }

    /// Send an SMTP command
    pub fn send_command(&mut self, command: ~str, option: Option<~str>) -> SmtpResponse {
        self.send_and_get_response(SmtpCommand::new(command, option).get_formatted_command())
    }

    /// Send an email
    pub fn send_message(&mut self, message: ~str) -> SmtpResponse {
        self.send_and_get_response(format!("{}{}.", message, CRLF))
    }

    /// Send a complete message or a command to the server and get the response
    fn send_and_get_response(&mut self, string: ~str) -> SmtpResponse {
        self.send(format!("{}{}", string, CRLF));

        match self.get_reply() {
            None => fail!("No answer on {}", self.host),
            Some(response) => response
        }
    }

    /// Send a string on the client socket
    fn send(&mut self, string: ~str) {
        self.socket.clone().unwrap().write_str(string);
        debug!("{:s}", string);
    }

    /// Read a string from the client socket
    fn read(&mut self) -> ~str {
        self.buf = [0u8, ..1000];

        let response = match self.socket.clone().unwrap().read(self.buf) {
            Err(error) => fail!("Read error"),
            Ok(bytes_read) => from_utf8(self.buf.slice_to(bytes_read - 1)).unwrap()
        };

        debug!("{:s}", response);

        return response.to_owned();
    }

    /// Get the SMTP response
    fn get_reply(&mut self) -> Option<SmtpResponse> {
        let response = self.read();

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
    pub fn new(host: &str, port: Option<Port>, my_hostname: Option<&str>) -> SmtpClient {
        SmtpClient{
            socket: None,
            host: host.to_owned(),
            port: port.unwrap_or(SMTP_PORT),
            my_hostname: my_hostname.unwrap_or("localhost").to_owned(),
            buf: [0u8, ..1000]
        }
    }

    /// Send an email
    pub fn send_mail(&mut self, from_addr: &str, to_addrs: &[&str], message: &str) {
        let my_hostname = self.my_hostname.clone();
        let mut server_response: SmtpResponse = self.connect();
        server_response = self.send_command(~"HELO", Some(my_hostname));
        server_response= self.send_command(~"MAIL", Some(from_addr.to_owned()));
        for &to_addr in to_addrs.iter() {
            server_response = self.send_command(~"RCPT", Some(to_addr.to_owned()));
        }
        server_response = self.send_command(~"DATA", None);
        server_response = self.send_message(message.to_owned());
        server_response = self.send_command(~"QUIT", None);
    }
}
