//! Error and result type for emails

use lettre;
use std::io;

/// An enum of all error kinds.
#[derive(Debug, Fail)]
pub enum Error {
    /// Envelope error
    #[fail(display = "lettre error: {}", error)]
    Envelope {
        /// inner error
        error: lettre::error::Error,
    },
    /// Unparseable filename for attachment
    #[fail(display = "the attachment filename could not be parsed")]
    CannotParseFilename,
    /// IO error
    #[fail(display = "IO error: {}", error)]
    Io {
        /// inner error
        error: io::Error,
    },
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io { error: err }
    }
}

impl From<lettre::error::Error> for Error {
    fn from(err: lettre::error::Error) -> Error {
        Error::Envelope { error: err }
    }
}
