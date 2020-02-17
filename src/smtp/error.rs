//! Error and result type for SMTP clients

use self::Error::*;
use crate::smtp::response::{Response, Severity};
use base64::DecodeError;
#[cfg(feature = "native-tls")]
use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io;
use std::string::FromUtf8Error;

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
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            // Try to display the first line of the server's response that usually
            // contains a short humanly readable error message
            Transient(ref err) => match err.first_line() {
                Some(line) => write!(f, "{}", line),
                None => write!(f, "undetailed transient error during SMTP transaction"),
            },
            Permanent(ref err) => match err.first_line() {
                Some(line) => write!(f, "{}", line),
                None => write!(f, "undetailed permanent error during SMTP transaction"),
            },
            ResponseParsing(err) => write!(f, "{}", err),
            ChallengeParsing(ref err) => write!(f, "{}", err),
            Utf8Parsing(ref err) => write!(f, "{}", err),
            Resolution => write!(f, "could not resolve hostname"),
            Client(err) => write!(f, "{}", err),
            Io(ref err) => write!(f, "{}", err),
            #[cfg(feature = "native-tls")]
            Tls(ref err) => write!(f, "{}", err),
            Parsing(ref err) => write!(f, "{}", err.description()),
            #[cfg(feature = "rustls-tls")]
            InvalidDNSName(ref err) => write!(f, "{}", err),
        }
    }
}

impl StdError for Error {
    fn cause(&self) -> Option<&dyn StdError> {
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

/// SMTP result type
pub type SmtpResult = Result<Response, Error>;
