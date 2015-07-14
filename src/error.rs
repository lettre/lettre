// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Error and result type for SMTP clients

use std::error::Error as StdError;
use std::io;
use std::fmt::{Display, Formatter};
use std::fmt;

use response::{Severity, Response};
use self::Error::*;

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// Transient SMTP error, 4xx reply code
    ///
    /// [RFC 5321, section 4.2.1](https://tools.ietf.org/html/rfc5321#section-4.2.1)
    TransientError(Response),
    /// Permanent SMTP error, 5xx reply code
    ///
    /// [RFC 5321, section 4.2.1](https://tools.ietf.org/html/rfc5321#section-4.2.1)
    PermanentError(Response),
    /// Error parsing a response
    ResponseParsingError(&'static str),
    /// Internal client error
    ClientError(&'static str),
    /// DNS resolution error
    ResolutionError,
    /// IO error
    IoError(io::Error),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            TransientError(_) => "a transient error occured during the SMTP transaction",
            PermanentError(_) => "a permanent error occured during the SMTP transaction",
            ResponseParsingError(_) => "an error occured while parsing an SMTP response",
            ResolutionError => "Could no resolve hostname",
            ClientError(_) => "an unknown error occured",
            IoError(_) => "an I/O error occured",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            IoError(ref err) => Some(&*err as &StdError),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        IoError(err)
    }
}

impl From<Response> for Error {
    fn from(response: Response) -> Error {
        match response.severity() {
            Severity::TransientNegativeCompletion => TransientError(response),
            Severity::PermanentNegativeCompletion => PermanentError(response),
            _ => ClientError("Unknown error code")
        }
    }
}

impl From<&'static str> for Error {
    fn from(string: &'static str) -> Error {
        ClientError(string)
    }
}

/// SMTP result type
pub type SmtpResult = Result<Response, Error>;

#[cfg(test)]
mod test {
    // TODO
}
