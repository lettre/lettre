//! SMTP client
//!
//! `SmtpConnection` allows manually sending SMTP commands.
//!
//! ```rust,no_run
//! # use std::error::Error;
//!
//! # #[cfg(feature = "smtp-transport")]
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use lettre::transport::smtp::{
//!     SMTP_PORT, client::SmtpConnection, commands::*, extension::ClientId,
//! };
//!
//! let hello = ClientId::Domain("my_hostname".to_owned());
//! let mut client = SmtpConnection::connect(&("localhost", SMTP_PORT), None, &hello, None, None)?;
//! client.command(Mail::new(Some("user@example.com".parse()?), vec![]))?;
//! client.command(Rcpt::new("user@example.org".parse()?, vec![]))?;
//! client.command(Data)?;
//! client.message("Test email".as_bytes())?;
//! client.command(Quit)?;
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "serde")]
use std::fmt::Debug;

#[cfg(any(feature = "tokio1", feature = "async-std1"))]
pub use self::async_connection::AsyncSmtpConnection;
#[cfg(any(feature = "tokio1", feature = "async-std1"))]
#[allow(deprecated)]
pub use self::async_net::AsyncNetworkStream;
#[cfg(feature = "tokio1")]
pub use self::async_net::AsyncTokioStream;
use self::net::NetworkStream;
#[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
pub(super) use self::tls::InnerTlsParameters;
#[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
pub use self::tls::TlsVersion;
pub use self::{
    connection::SmtpConnection,
    tls::{Certificate, CertificateStore, Identity, Tls, TlsParameters, TlsParametersBuilder},
};

#[cfg(any(feature = "tokio1", feature = "async-std1"))]
mod async_connection;
#[cfg(any(feature = "tokio1", feature = "async-std1"))]
mod async_net;
mod connection;
mod net;
mod tls;

/// Total bytes cap on an SMTP response (Postfix `smtp_response_limit`).
///
/// There is no separate cap on the length of a single line: some servers
/// (e.g. Microsoft 365) send reply lines well over the 512 bytes of
/// RFC 5321, and every line read is bounded by what is left of this cap.
pub(super) const MAX_RESPONSE_BYTES: usize = 100_000;

/// The codec used for transparency
#[derive(Debug)]
struct ClientCodec {
    status: CodecStatus,
}

impl ClientCodec {
    /// Creates a new client codec
    pub(crate) fn new() -> Self {
        Self {
            status: CodecStatus::StartOfNewLine,
        }
    }

    /// Adds transparency ("dot stuffing"), as required by RFC 5321 section 4.5.2.
    ///
    /// A line of the message body that starts with a `.` gets a second `.`
    /// prepended. Besides CRLF, a bare CR or a bare LF is also treated as a line
    /// boundary, so that a body which was not normalized to CRLF (e.g. raw bytes
    /// containing Unix line endings) cannot smuggle an unstuffed `.` past a
    /// server that accepts bare line endings as line terminators.
    fn encode(&mut self, frame: &[u8], buf: &mut Vec<u8>) {
        for &b in frame {
            if b == b'.' && !matches!(self.status, CodecStatus::MiddleOfLine) {
                buf.push(b'.');
            }
            buf.push(b);
            self.status = match (b, self.status) {
                (b'\r', _) => CodecStatus::StartingNewLine,
                (b'\n', CodecStatus::StartingNewLine) => CodecStatus::StartOfNewLine,
                (b'\n', _) => CodecStatus::AfterBareLf,
                _ => CodecStatus::MiddleOfLine,
            };
        }
    }

    /// Returns the sequence ending the mail data, whose first CRLF ends an unterminated last line
    ///
    /// Only a CRLF ends the last line here: a server that doesn't take a bare
    /// CR or LF for a line ending is still in the middle of a line after one.
    fn terminator(&self) -> &'static [u8] {
        match self.status {
            CodecStatus::StartOfNewLine => b".\r\n",
            CodecStatus::MiddleOfLine | CodecStatus::StartingNewLine | CodecStatus::AfterBareLf => {
                b"\r\n.\r\n"
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum CodecStatus {
    /// We are past the first character of the current line
    MiddleOfLine,
    /// We just read a `\r` character
    StartingNewLine,
    /// We just read a `\n` character that wasn't preceded by a `\r`
    AfterBareLf,
    /// We are at the start of a new line, after a CRLF
    StartOfNewLine,
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
        let mut buf = Vec::new();
        let mut codec = ClientCodec::new();

        codec.encode(b".\r\n", &mut buf);
        codec.encode(b"test\r\n", &mut buf);
        codec.encode(b"test\r\n\r\n", &mut buf);
        codec.encode(b".\r\n", &mut buf);
        codec.encode(b"\r\ntest", &mut buf);
        codec.encode(b"te\r\n.\r\nst", &mut buf);
        codec.encode(b"test", &mut buf);
        codec.encode(b"test.", &mut buf);
        codec.encode(b"test\n", &mut buf);
        codec.encode(b".test\n", &mut buf);
        codec.encode(b"test", &mut buf);
        codec.encode(b"test", &mut buf);
        codec.encode(b"test\r\n", &mut buf);
        codec.encode(b".test\r\n", &mut buf);
        codec.encode(b"test.\r\n", &mut buf);
        assert_eq!(
            String::from_utf8(buf).unwrap(),
            "..\r\ntest\r\ntest\r\n\r\n..\r\n\r\ntestte\r\n..\r\nsttesttest.test\n..test\ntesttesttest\r\n..test\r\ntest.\r\n"
        );
    }

    #[test]
    fn test_codec_terminator() {
        let terminator = |parts: &[&[u8]]| {
            let mut codec = ClientCodec::new();
            for part in parts {
                codec.encode(part, &mut Vec::new());
            }
            codec.terminator()
        };

        assert_eq!(terminator(&[]), b".\r\n");
        assert_eq!(terminator(&[b"test\r\n"]), b".\r\n");
        assert_eq!(terminator(&[b"test\r", b"\n"]), b".\r\n");
        assert_eq!(terminator(&[b"test"]), b"\r\n.\r\n");
        assert_eq!(terminator(&[b"test\r"]), b"\r\n.\r\n");
        assert_eq!(terminator(&[b"test\n"]), b"\r\n.\r\n");
        // A bare CR or LF starts a line as far as dots go, but it doesn't end
        // the last line of the data
        assert_eq!(terminator(&[b"test\r\n\n"]), b"\r\n.\r\n");
        assert_eq!(terminator(&[b"test\r\n\r"]), b"\r\n.\r\n");
        assert_eq!(terminator(&[b"test\n\r\n"]), b".\r\n");
        assert_eq!(terminator(&[b"test\r\r\n"]), b".\r\n");
    }

    #[test]
    fn test_codec_bare_lf_is_dot_stuffed() {
        // A `.` at the start of a line delimited by a bare LF (or bare CR) must
        // still be dot-stuffed, otherwise `\n.\n` could be interpreted as the
        // end of DATA by servers that accept bare line endings (SMTP smuggling).
        let mut buf = Vec::new();
        let mut codec = ClientCodec::new();
        codec.encode(b"hello\n.\nworld", &mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "hello\n..\nworld");

        let mut buf = Vec::new();
        let mut codec = ClientCodec::new();
        codec.encode(b"hello\r.\rworld", &mut buf);
        assert_eq!(String::from_utf8(buf).unwrap(), "hello\r..\rworld");
    }

    #[test]
    #[cfg(feature = "tracing")]
    fn test_escape_crlf() {
        assert_eq!(escape_crlf("\r\n"), "<CRLF>");
        assert_eq!(escape_crlf("EHLO my_name\r\n"), "EHLO my_name<CRLF>");
        assert_eq!(
            escape_crlf("EHLO my_name\r\nSIZE 42\r\n"),
            "EHLO my_name<CRLF>SIZE 42<CRLF>"
        );
    }
}
