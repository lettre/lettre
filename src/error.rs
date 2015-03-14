// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Error and result type for SMTP clients

use std::error::Error;
use std::io;
use std::error::FromError;
use std::fmt::{Display, Formatter};
use std::fmt;

use response::{Severity, Response};
use self::SmtpError::*;

/// An enum of all error kinds.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SmtpError {
    /// Transient error, 4xx reply code
    ///
    /// [RFC 5321, section 4.2.1](https://tools.ietf.org/html/rfc5321#section-4.2.1)
    TransientError(Response),
    /// Permanent error, 5xx reply code
    ///
    /// [RFC 5321, section 4.2.1](https://tools.ietf.org/html/rfc5321#section-4.2.1)
    PermanentError(Response),
    /// TODO
    ClientError(String),
    /// IO error
    IoError(io::Error),
}

impl Display for SmtpError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.write_str(self.description())
    }
}

impl Error for SmtpError {
    fn description(&self) -> &str {
        match *self {
            TransientError(_) => "a transient error occured during the SMTP transaction",
            PermanentError(_) => "a permanent error occured during the SMTP transaction",
            ClientError(_) => "an unknown error occured",
            IoError(_) => "an I/O error occured",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            IoError(ref err) => Some(&*err as &Error),
            _ => None,
        }
    }
}

impl FromError<io::Error> for SmtpError {
    fn from_error(err: io::Error) -> SmtpError {
        IoError(err)
    }
}

impl FromError<Response> for SmtpError {
    fn from_error(response: Response) -> SmtpError {
        match response.severity() {
            Severity::TransientNegativeCompletion => TransientError(response),
            Severity::PermanentNegativeCompletion => PermanentError(response),
            _ => ClientError("Unknown error code".to_string())
        }
    }
}

impl FromError<&'static str> for SmtpError {
    fn from_error(string: &'static str) -> SmtpError {
        ClientError(string.to_string())
    }
}

/// SMTP result type
pub type SmtpResult = Result<Response, SmtpError>;

#[cfg(test)]
mod test {
    // TODO
}
