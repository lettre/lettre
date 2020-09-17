//! SMTP client
//!
//! `SmtpConnection` allows manually sending SMTP commands.
//!
//! ```rust,no_run
//! # #[cfg(feature = "smtp-transport")]
//! # {
//! use lettre::transport::smtp::{SMTP_PORT, extension::ClientId, commands::*, client::SmtpConnection};
//!
//! let hello = ClientId::Domain("my_hostname".to_string());
//! let mut client = SmtpConnection::connect(&("localhost", SMTP_PORT), None, &hello, None).unwrap();
//! client.command(
//!         Mail::new(Some("user@example.com".parse().unwrap()), vec![])
//!     ).unwrap();
//! client.command(
//!         Rcpt::new("user@example.org".parse().unwrap(), vec![])
//!       ).unwrap();
//! client.command(Data).unwrap();
//! client.message("Test email".as_bytes()).unwrap();
//! client.command(Quit).unwrap();
//! # }
//! ```

#[cfg(feature = "serde")]
use std::fmt::Debug;

#[cfg(feature = "tokio02")]
pub(crate) use self::async_connection::AsyncSmtpConnection;
#[cfg(feature = "tokio02")]
pub(crate) use self::async_net::AsyncNetworkStream;
use self::net::NetworkStream;
#[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
pub(super) use self::tls::InnerTlsParameters;
pub use self::{
    connection::SmtpConnection,
    mock::MockStream,
    tls::{Tls, TlsParameters, TlsParametersBuilder},
};

#[cfg(feature = "tokio02")]
mod async_connection;
#[cfg(feature = "tokio02")]
mod async_net;
mod connection;
mod mock;
mod net;
mod tls;

/// The codec used for transparency
#[derive(Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientCodec {
    escape_count: u8,
}

impl ClientCodec {
    /// Creates a new client codec
    pub fn new() -> Self {
        ClientCodec::default()
    }

    /// Adds transparency
    fn encode(&mut self, frame: &[u8], buf: &mut Vec<u8>) {
        match frame.len() {
            0 => {
                match self.escape_count {
                    0 => buf.extend_from_slice(b"\r\n.\r\n"),
                    1 => buf.extend_from_slice(b"\n.\r\n"),
                    2 => buf.extend_from_slice(b".\r\n"),
                    _ => unreachable!(),
                }
                self.escape_count = 0;
            }
            _ => {
                let mut start = 0;
                for (idx, byte) in frame.iter().enumerate() {
                    match self.escape_count {
                        0 => self.escape_count = if *byte == b'\r' { 1 } else { 0 },
                        1 => self.escape_count = if *byte == b'\n' { 2 } else { 0 },
                        2 => self.escape_count = if *byte == b'.' { 3 } else { 0 },
                        _ => unreachable!(),
                    }
                    if self.escape_count == 3 {
                        self.escape_count = 0;
                        buf.extend_from_slice(&frame[start..idx]);
                        buf.extend_from_slice(b".");
                        start = idx;
                    }
                }
                buf.extend_from_slice(&frame[start..]);
            }
        }
    }
}

/// Returns the string replacing all the CRLF with "\<CRLF\>"
/// Used for debug displays
#[cfg(feature = "tracing")]
pub(super) fn escape_crlf(string: &str) -> String {
    string.replace("\r\n", "<CRLF>")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_codec() {
        let mut codec = ClientCodec::new();
        let mut buf: Vec<u8> = vec![];

        codec.encode(b"test\r\n", &mut buf);
        codec.encode(b".\r\n", &mut buf);
        codec.encode(b"\r\ntest", &mut buf);
        codec.encode(b"te\r\n.\r\nst", &mut buf);
        codec.encode(b"test", &mut buf);
        codec.encode(b"test.", &mut buf);
        codec.encode(b"test\n", &mut buf);
        codec.encode(b".test\n", &mut buf);
        codec.encode(b"test", &mut buf);
        assert_eq!(
            String::from_utf8(buf).unwrap(),
            "test\r\n..\r\n\r\ntestte\r\n..\r\nsttesttest.test\n.test\ntest"
        );
    }

    #[test]
    #[cfg(feature = "log")]
    fn test_escape_crlf() {
        assert_eq!(escape_crlf("\r\n"), "<CRLF>");
        assert_eq!(escape_crlf("EHLO my_name\r\n"), "EHLO my_name<CRLF>");
        assert_eq!(
            escape_crlf("EHLO my_name\r\nSIZE 42\r\n"),
            "EHLO my_name<CRLF>SIZE 42<CRLF>"
        );
    }
}
