//! Error and result type for file transport

use self::Error::*;
use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Display, Formatter};

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Internal client error
    Client(&'static str),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Client(_) => "an unknown error occured",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

impl From<&'static str> for Error {
    fn from(string: &'static str) -> Error {
        Client(string)
    }
}
