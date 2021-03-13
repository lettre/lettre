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
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match &self {
            Client(err) => err.fmt(fmt),
            Utf8Parsing(err) => err.fmt(fmt),
            Io(err) => err.fmt(fmt),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self {
            Io(err) => Some(&*err),
            Utf8Parsing(err) => Some(&*err),
            _ => None,
        }
    }
}
