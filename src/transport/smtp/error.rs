//! Error and result type for SMTP clients

use self::Error::*;
use crate::transport::smtp::response::{Response, Severity};
use base64::DecodeError;
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io,
    string::FromUtf8Error,
};

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Transient SMTP error, 4xx reply code
    ///
    /// [RFC 5321, section 4.2.1](https://tools.ietf.org/html/rfc5321#section-4.2.1)
    Transient(Response),
    /// Permanent SMTP error, 5xx reply code
    ///
    /// [RFC 5321, section 4.2.1](https://tools.ietf.org/html/rfc5321#section-4.2.1)
    Permanent(Response),
    /// Error parsing a response
    ResponseParsing(&'static str),
    /// Error parsing a base64 string in response
    ChallengeParsing(DecodeError),
    /// Error parsing UTF8in response
    Utf8Parsing(FromUtf8Error),
    /// Internal client error
    Client(&'static str),
    /// DNS resolution error
    Resolution,
    /// IO error
    Io(io::Error),
    /// TLS error
    #[cfg(feature = "native-tls")]
    Tls(native_tls::Error),
    /// Parsing error
    Parsing(nom::error::ErrorKind),
    /// Invalid hostname
    #[cfg(feature = "rustls-tls")]
    InvalidDNSName(webpki::InvalidDNSNameError),
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    InvalidCertificate,
    #[cfg(feature = "r2d2")]
    Pool(r2d2::Error),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            // Try to display the first line of the server's response that usually
            // contains a short humanly readable error message
            Transient(ref err) => fmt.write_str(match err.first_line() {
                Some(line) => line,
                None => "transient error during SMTP transaction",
            }),
            Permanent(ref err) => fmt.write_str(match err.first_line() {
                Some(line) => line,
                None => "permanent error during SMTP transaction",
            }),
            ResponseParsing(err) => fmt.write_str(err),
            ChallengeParsing(ref err) => err.fmt(fmt),
            Utf8Parsing(ref err) => err.fmt(fmt),
            Resolution => fmt.write_str("could not resolve hostname"),
            Client(err) => fmt.write_str(err),
            Io(ref err) => err.fmt(fmt),
            #[cfg(feature = "native-tls")]
            Tls(ref err) => err.fmt(fmt),
            Parsing(ref err) => fmt.write_str(err.description()),
            #[cfg(feature = "rustls-tls")]
            InvalidDNSName(ref err) => err.fmt(fmt),
            #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
            InvalidCertificate => fmt.write_str("invalid certificate"),
            #[cfg(feature = "r2d2")]
            Pool(ref err) => err.fmt(fmt),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            ChallengeParsing(ref err) => Some(&*err),
            Utf8Parsing(ref err) => Some(&*err),
            Io(ref err) => Some(&*err),
            #[cfg(feature = "native-tls")]
            Tls(ref err) => Some(&*err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Io(err)
    }
}

#[cfg(feature = "native-tls")]
impl From<native_tls::Error> for Error {
    fn from(err: native_tls::Error) -> Error {
        Tls(err)
    }
}

impl From<nom::Err<(&str, nom::error::ErrorKind)>> for Error {
    fn from(err: nom::Err<(&str, nom::error::ErrorKind)>) -> Error {
        Parsing(match err {
            nom::Err::Incomplete(_) => nom::error::ErrorKind::Complete,
            nom::Err::Failure((_, k)) => k,
            nom::Err::Error((_, k)) => k,
        })
    }
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Error {
        ChallengeParsing(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Utf8Parsing(err)
    }
}

#[cfg(feature = "rustls-tls")]
impl From<webpki::InvalidDNSNameError> for Error {
    fn from(err: webpki::InvalidDNSNameError) -> Error {
        InvalidDNSName(err)
    }
}

#[cfg(feature = "r2d2")]
impl From<r2d2::Error> for Error {
    fn from(err: r2d2::Error) -> Error {
        Pool(err)
    }
}

impl From<Response> for Error {
    fn from(response: Response) -> Error {
        match response.code.severity {
            Severity::TransientNegativeCompletion => Transient(response),
            Severity::PermanentNegativeCompletion => Permanent(response),
            _ => Client("Unknown error code"),
        }
    }
}

impl From<&'static str> for Error {
    fn from(string: &'static str) -> Error {
        Client(string)
    }
}
