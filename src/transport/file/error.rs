//! Error and result type for file transport

use self::Error::*;
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io,
};

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Internal client error
    Client(&'static str),
    /// IO error
    Io(io::Error),
    /// JSON error
    #[cfg(feature = "file-transport-envelope")]
    Json(serde_json::Error),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            Client(err) => fmt.write_str(err),
            Io(ref err) => err.fmt(fmt),
            #[cfg(feature = "file-transport-envelope")]
            Json(ref err) => err.fmt(fmt),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Io(ref err) => Some(&*err),
            #[cfg(feature = "file-transport-envelope")]
            Json(ref err) => Some(&*err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

#[cfg(feature = "file-transport-envelope")]
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}

impl From<&'static str> for Error {
    fn from(string: &'static str) -> Error {
        Error::Client(string)
    }
}
