//! Error and result type for emails

use self::Error::*;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io;

use lettre;

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Envelope error
    Email(lettre::Error),
    /// Unparseable filename for attachment
    CannotParseFilename,
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
            Email(ref err) => err.description(),
            CannotParseFilename => "the attachment filename could not be parsed",
            Io(ref err) => err.description(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Io(err)
    }
}

impl From<lettre::Error> for Error {
    fn from(err: lettre::Error) -> Error {
        Email(err)
    }
}
