//! Error and result type for emails

use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Display, Formatter};

use self::Error::*;

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Missinf sender
    MissingFrom,
    /// Missing recipient
    MissingTo,
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            MissingFrom => "the sender is missing",
            MissingTo => "the recipient is missing",
        }
    }
}
