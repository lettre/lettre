//! Error and result type for sendmail transport

use self::Error::*;
use std::io;
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Internal client error
    Client(&'static str),
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
            Client(err) => err,
            Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Io(ref err) => Some(&*err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<&'static str> for Error {
    fn from(string: &'static str) -> Error {
        Error::Client(string)
    }
}

/// sendmail result type
pub type SendmailResult = Result<(), Error>;
