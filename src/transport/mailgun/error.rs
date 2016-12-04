//! Error and result type for mailgun transport


use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Display, Formatter};

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Serde encoding error
    Serde(::serde_urlencoded::ser::Error),
    /// Hyper send error
    Hyper(::hyper::Error),
    /// Mailgun Error
    Mailgun(::hyper::client::Response),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Serde(_) => "Serde encoding encountered an error",
            Error::Hyper(_) => "Hyper sending encountered an error",
            Error::Mailgun(_) => "Mailgun could not send the email",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Serde(ref e) => Some(&*e as &StdError),
            Error::Hyper(ref e) => Some(&*e as &StdError),
            _ => None,
        }
    }
}

impl From<::serde_urlencoded::ser::Error> for Error {
    fn from(err: ::serde_urlencoded::ser::Error) -> Error {
        Error::Serde(err)
    }
}

impl From<::hyper::Error> for Error {
    fn from(err: ::hyper::Error) -> Error {
        Error::Hyper(err)
    }
}


/// mailgun result type
pub type MailgunResult<T> = Result<T, Error>;

