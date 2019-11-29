//! Error and result type for sendmail transport

use self::Error::*;
use std::io;
use std::string::FromUtf8Error;
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Internal client error
    Client(String),
    /// Error parsing UTF8in response
    Utf8Parsing(FromUtf8Error),
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
            Client(ref err) => err,
            Utf8Parsing(ref err) => err.description(),
            Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
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
