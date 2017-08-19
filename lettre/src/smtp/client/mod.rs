//! SMTP client

use bufstream::BufStream;
use smtp::{CRLF, MESSAGE_ENDING};
use smtp::authentication::{Credentials, Mechanism};
use smtp::client::net::{ClientTlsParameters, Connector, NetworkStream, Timeout};
use smtp::commands::*;
use smtp::error::{Error, SmtpResult};
use smtp::response::ResponseParser;
use std::fmt::Debug;
use std::fmt::Display;
use std::io;
use std::io::{BufRead, Read, Write};
use std::net::ToSocketAddrs;
use std::string::String;
use std::time::Duration;
use std::io::BufReader;


pub mod net;
pub mod mock;

/// The codec used to encode client requests and decode server responses
#[derive(Default,Debug)]
pub struct ClientCodec {
    escape_count: u8,
}

impl ClientCodec {
    pub fn new() -> Self {
        ClientCodec::default()
    }
}

impl ClientCodec {
    // TODO replace CR and LF by CRLF
    fn encode(&mut self, frame: &[u8], buf: &mut Vec<u8>) -> Result<(), Error> {
        match frame.len() {
            0 => {
                match self.escape_count {
                    0 => buf.write_all(b"\r\n.\r\n")?,
                    1 => buf.write_all(b"\n.\r\n")?,
                    2 => buf.write_all(b".\r\n")?,
                    _ => unreachable!(),
                }
                self.escape_count = 0;
                Ok(())
            },
            _ => {
                let mut start = 0;
                for (idx, byte) in frame.iter().enumerate() {
                    match self.escape_count {
                        0 => self.escape_count = if *byte == b'\r' { 1 } else { 0 },
                        1 => self.escape_count = if *byte == b'\n' { 2 } else { 0 },
                        2 => self.escape_count = if *byte == b'.'  { 3 } else { 0 },
                        _ => unreachable!(),
                    }
                    if self.escape_count == 3 {
                        self.escape_count = 0;
                        buf.write_all(&frame[start..idx])?;
                        buf.write_all(b".")?;
                        start = idx;
                    }
                }
                Ok(buf.write_all(&frame[start..])?)
            },

        }
    }
}

/// Returns the string replacing all the CRLF with "\<CRLF\>"
/// Used for debug displays
fn escape_crlf(string: &str) -> String {
    string.replace(CRLF, "<CRLF>")
}

/// Returns the string removing all the CRLF
/// Used for debug displays
fn remove_crlf(string: &str) -> String {
    string.replace(CRLF, "")
}

/// Structure that implements the SMTP client
#[derive(Debug, Default)]
pub struct Client<S: Write + Read = NetworkStream> {
    /// TCP stream between client and server
    /// Value is None before connection
    stream: Option<BufStream<S>>,
}

macro_rules! return_err (
    ($err: expr, $client: ident) => ({
        return Err(From::from($err))
    })
);

#[cfg_attr(feature = "cargo-clippy", allow(new_without_default_derive))]
impl<S: Write + Read> Client<S> {
    /// Creates a new SMTP client
    ///
    /// It does not connects to the server, but only creates the `Client`
    pub fn new() -> Client<S> {
        Client { stream: None }
    }
}

impl<S: Connector + Write + Read + Timeout + Debug> Client<S> {
    /// Closes the SMTP transaction if possible
    pub fn close(&mut self) {
        let _ = self.smtp_command(QuitCommand);
        self.stream = None;
    }

    /// Sets the underlying stream
    pub fn set_stream(&mut self, stream: S) {
        self.stream = Some(BufStream::new(stream));
    }

    /// Upgrades the underlying connection to SSL/TLS
    pub fn upgrade_tls_stream(&mut self, tls_parameters: &ClientTlsParameters) -> io::Result<()> {
        match self.stream {
            Some(ref mut stream) => stream.get_mut().upgrade_tls(tls_parameters),
            None => Ok(()),
        }
    }

    /// Tells if the underlying stream is currently encrypted
    pub fn is_encrypted(&self) -> bool {
        match self.stream {
            Some(ref stream) => stream.get_ref().is_encrypted(),
            None => false,
        }
    }

    /// Set timeout
    pub fn set_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        match self.stream {
            Some(ref mut stream) => {
                stream.get_mut().set_read_timeout(duration)?;
                stream.get_mut().set_read_timeout(duration)?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    /// Connects to the configured server
    pub fn connect<A: ToSocketAddrs>(
        &mut self,
        addr: &A,
        tls_parameters: Option<&ClientTlsParameters>,
    ) -> SmtpResult {
        // Connect should not be called when the client is already connected
        if self.stream.is_some() {
            return_err!("The connection is already established", self);
        }

        let mut addresses = addr.to_socket_addrs()?;

        let server_addr = match addresses.next() {
            Some(addr) => addr,
            None => return_err!("Could not resolve hostname", self),
        };

        debug!("connecting to {}", server_addr);

        // Try to connect
        self.set_stream(Connector::connect(&server_addr, tls_parameters)?);

        self.get_reply()
    }

    /// Checks if the server is connected using the NOOP SMTP command
    #[cfg_attr(feature = "cargo-clippy", allow(wrong_self_convention))]
    pub fn is_connected(&mut self) -> bool {
        self.smtp_command(NoopCommand).is_ok()
    }

    /// Sends an AUTH command with the given mechanism, and handles challenge if needed
    pub fn auth(&mut self, mechanism: Mechanism, credentials: &Credentials) -> SmtpResult {
        // TODO
        let mut challenges = 10;
        let mut response = self.smtp_command(
            AuthCommand::new(mechanism, credentials.clone(), None)?,
        )?;

        while challenges > 0 && response.has_code(334) {
            challenges -= 1;
            response = self.smtp_command(AuthCommand::new_from_response(
                mechanism,
                credentials.clone(),
                response,
            )?)?;
        }

        if challenges == 0 {
            Err(Error::ResponseParsing("Unexpected number of challenges"))
        } else {
            Ok(response)
        }
    }

    /// Sends the message content
    pub fn message<T: Read>(&mut self, mut message: Box<T>) -> SmtpResult {
        let mut in_buf: Vec<u8> = vec![];
        let mut out_buf: Vec<u8> = vec![];

        let mut codec = ClientCodec::new();
        let mut message_reader = BufReader::new(message.as_mut());

        loop {
            in_buf.clear();
            out_buf.clear();
            match message_reader.read(&mut in_buf)? {
                0 => break,
                _ => codec.encode(in_buf.as_slice(), &mut out_buf)?,
            };

            self.write_server(out_buf.as_slice())?;
        }

        self.write_server(MESSAGE_ENDING.as_bytes())?;
        self.get_reply()
    }

    /// Sends an SMTP command
    pub fn smtp_command<C: Display>(&mut self, command: C) -> SmtpResult {
        self.write_server(command.to_string().as_bytes())?;
        self.get_reply()
    }

    /// Writes a string to the server
    fn write_server(&mut self, string: &[u8]) -> Result<(), Error> {
        if self.stream.is_none() {
            return Err(From::from("Connection closed"));
        }

        self.stream.as_mut().unwrap().write(string)?;
        self.stream.as_mut().unwrap().flush()?;

        debug!("Wrote: {}", escape_crlf(String::from_utf8_lossy(string).as_ref()));
        Ok(())
    }

    /// Gets the SMTP response
    fn get_reply(&mut self) -> SmtpResult {

        let mut parser = ResponseParser::default();

        let mut line = String::new();
        self.stream.as_mut().unwrap().read_line(&mut line)?;

        debug!("Read: {}", escape_crlf(line.as_ref()));

        while parser.read_line(remove_crlf(line.as_ref()).as_ref())? {
            line.clear();
            self.stream.as_mut().unwrap().read_line(&mut line)?;
        }

        let response = parser.response()?;

        if response.is_positive() {
            Ok(response)
        } else {
            Err(From::from(response))
        }
    }
}

#[cfg(test)]
mod test {
    use super::{escape_crlf, remove_crlf, ClientCodec};

    #[test]
    fn test_codec() {
        let mut codec = ClientCodec::new();
        let mut buf : Vec<u8> = vec![];

        codec.encode(b"test\r\n", &mut buf);
        codec.encode(b".\r\n", &mut buf);
        codec.encode(b"\r\ntest", &mut buf);
        codec.encode(b"te\r\n.\r\nst", &mut buf);
        codec.encode(b"test", &mut buf);
        codec.encode(b"test.", &mut buf);
        codec.encode(b"test\n", &mut buf);
        codec.encode(b".test\n", &mut buf);
        codec.encode(b"test", &mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "test\r\n..\r\n\r\ntestte\r\n..\r\nsttesttest.test\n.test\ntest");
    }  

    #[test]
    fn test_remove_crlf() {
        assert_eq!(remove_crlf("\r\n"), "");
        assert_eq!(remove_crlf("EHLO my_name\r\n"), "EHLO my_name");
        assert_eq!(
            remove_crlf("EHLO my_name\r\nSIZE 42\r\n"),
            "EHLO my_nameSIZE 42"
        );
    }

    #[test]
    fn test_escape_crlf() {
        assert_eq!(escape_crlf("\r\n"), "<CRLF>");
        assert_eq!(escape_crlf("EHLO my_name\r\n"), "EHLO my_name<CRLF>");
        assert_eq!(
            escape_crlf("EHLO my_name\r\nSIZE 42\r\n"),
            "EHLO my_name<CRLF>SIZE 42<CRLF>"
        );
    }
}
