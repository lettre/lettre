//! Error and result type for emails

use self::Error::*;
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io,
};

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Envelope error
    Envelope(crate::error::Error),
    /// Unparseable filename for attachment
    CannotParseFilename,
    /// IO error
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        fmt.write_str(&match *self {
            CannotParseFilename => "Could not parse attachment filename".to_owned(),
            Io(ref err) => err.to_string(),
            Envelope(ref err) => err.to_string(),
        })
    }
}

impl StdError for Error {
    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            Envelope(ref err) => Some(err),
            Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<crate::error::Error> for Error {
    fn from(err: crate::error::Error) -> Error {
        Error::Envelope(err)
    }
}
