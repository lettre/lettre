//! Error and result type for sendmail transport

use failure;
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
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io { error: err }
    }
}

impl From<&'static str> for Error {
    fn from(string: &'static str) -> Error {
        Error::Client { error: string }
    }
}

/// sendmail result type
pub type SendmailResult = Result<(), failure::Error>;
