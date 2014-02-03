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
use common::SMTP_PORT;
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
        let ips = get_host_addresses(self.host.clone());
        let ip = ips.expect(format!("Cannot resolve {}", self.host))[0];

        match TcpStream::connect(SocketAddr{ip: ip, port: self.port}) {
            None => fail!("Cannot connect to {}:{}", self.host, self.port),
            Some(s) => self.socket = Some(s)
        }

        match self.get_reply() {
            None => fail!("No banner on {}", self.host),
            Some(response) => response
        }
    }

    /// Send an SMTP command
    pub fn send_command(&mut self, command: ~str, option: Option<~str>) -> SmtpResponse {

        self.send(SmtpCommand::new(command, option).get_formatted_command());
        let response = self.get_reply();

        match response {
            None => fail!("No answer on {}", self.host),
            Some(response) => response
        }
    }

    /// Send a string on the client socket
    fn send(&mut self, string: ~str) {
        self.socket.write_str(string);
        debug!("{:s}", string);
    }

    /// Get the SMTP response
    fn get_reply(&mut self) -> Option<SmtpResponse> {
        self.buf = [0u8, ..1000];

        let response = match self.socket.read(self.buf) {
            None => fail!("Read error"),
            Some(bytes_read) => self.buf.slice_to(bytes_read - 1)
        };

        debug!("{:s}", from_utf8(response).unwrap());

        if response.len() > 4 {
            Some(SmtpResponse {
                    code: from_str(from_utf8(response.slice_to(3)).unwrap()).unwrap(),
                    message: from_utf8(response.slice_from(4)).unwrap().to_owned()
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
        self.connect();
        self.send_command(~"HELO", Some(my_hostname));
        self.send_command(~"MAIL", Some(from_addr.to_owned()));
        for &to_addr in to_addrs.iter() {
            self.send_command(~"RCPT", Some(to_addr.to_owned()));
        }
        self.send_command(~"DATA", None);
        self.send(message.to_owned());
        self.send(~"\r\n.\r\n");
        self.send_command(~"QUIT", None);
    }
}
