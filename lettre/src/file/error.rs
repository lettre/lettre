//! Error and result type for file transport

use failure;
use serde_json;
use std::io;

/// An enum of all error kinds.
#[derive(Fail, Debug)]
pub enum Error {
    /// Internal client error
    #[fail(display = "Internal client error: {}", error)]
    Client { error: &'static str },
    /// IO error
    #[fail(display = "IO error: {}", error)]
    Io { error: io::Error },
    /// JSON serialization error
    #[fail(display = "JSON serialization error: {}", error)]
    JsonSerialization { error: serde_json::Error },
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io { error: err }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonSerialization { error: err }
    }
}

impl From<&'static str> for Error {
    fn from(string: &'static str) -> Error {
        Error::Client { error: string }
    }
}

/// SMTP result type
pub type FileResult = Result<(), failure::Error>;
