use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};
use self::Error::*;

/// Error type for email content
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// Missing from in envelope
    MissingFrom,
    /// Missing to in envelope
    MissingTo,
    /// Invalid email
    InvalidEmailAddress,
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        fmt.write_str(&match *self {
            MissingFrom => "missing source address, invalid envelope".to_owned(),
            MissingTo => "missing destination address, invalid envelope".to_owned(),
            InvalidEmailAddress => "invalid email address".to_owned(),
        })
    }
}

impl StdError for Error {
    fn cause(&self) -> Option<&dyn StdError> {
        None
    }
}

/// Email result type
pub type EmailResult<T> = Result<T, Error>;
