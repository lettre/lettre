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
    /// IO error
    Io(io::Error),
    /// JSON error
    #[cfg(feature = "file-transport-envelope")]
    Json(serde_json::Error),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match &self {
            Io(err) => err.fmt(fmt),
            #[cfg(feature = "file-transport-envelope")]
            Json(err) => err.fmt(fmt),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self {
            Io(err) => Some(&*err),
            #[cfg(feature = "file-transport-envelope")]
            Json(err) => Some(&*err),
        }
    }
}
