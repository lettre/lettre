//! Error and result type for sendmail transport

use self::Error::*;
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io,
    string::FromUtf8Error,
};

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Internal client error
    Client(String),
    /// Error parsing UTF8 in response
    Utf8Parsing(FromUtf8Error),
    /// IO error
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            Client(ref err) => err.fmt(fmt),
            Utf8Parsing(ref err) => err.fmt(fmt),
            Io(ref err) => err.fmt(fmt),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Io(ref err) => Some(&*err),
            Utf8Parsing(ref err) => Some(&*err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Utf8Parsing(err)
    }
}

/// sendmail result type
pub type SendmailResult = Result<(), Error>;
