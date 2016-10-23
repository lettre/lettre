//! Error and result type for SMTP clients

use rustc_serialize::base64::FromBase64Error;
use self::Error::*;
use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io;
use transport::smtp::response::{Response, Severity};

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
    ChallengeParsing(FromBase64Error),
    /// Internal client error
    Client(&'static str),
    /// DNS resolution error
    Resolution,
    /// IO error
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Transient(_) => "a transient error occured during the SMTP transaction",
            Permanent(_) => "a permanent error occured during the SMTP transaction",
            ResponseParsing(_) => "an error occured while parsing an SMTP response",
            ChallengeParsing(_) => "an error occured while parsing a CRAM-MD5 challenge",
            Resolution => "could not resolve hostname",
            Client(_) => "an unknown error occured",
            Io(_) => "an I/O error occured",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Io(ref err) => Some(&*err as &StdError),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Io(err)
    }
}

impl From<Response> for Error {
    fn from(response: Response) -> Error {
        match response.severity() {
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
