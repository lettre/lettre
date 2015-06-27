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
use std::fmt::{Display, Formatter};
use std::fmt;

use response::{Severity, Response};
use self::SmtpError::*;

/// An enum of all error kinds.
#[derive(Debug)]
pub enum SmtpError {
    /// Transient SMTP error, 4xx reply code
    ///
    /// [RFC 5321, section 4.2.1](https://tools.ietf.org/html/rfc5321#section-4.2.1)
    TransientError(Response),
    /// Permanent SMTP error, 5xx reply code
    ///
    /// [RFC 5321, section 4.2.1](https://tools.ietf.org/html/rfc5321#section-4.2.1)
    PermanentError(Response),
    /// Internal client error
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

impl From<io::Error> for SmtpError {
    fn from(err: io::Error) -> SmtpError {
        IoError(err)
    }
}

impl From<Response> for SmtpError {
    fn from(response: Response) -> SmtpError {
        match response.severity() {
            Severity::TransientNegativeCompletion => TransientError(response),
            Severity::PermanentNegativeCompletion => PermanentError(response),
            _ => ClientError("Unknown error code".to_string())
        }
    }
}

impl From<&'static str> for SmtpError {
    fn from(string: &'static str) -> SmtpError {
        ClientError(string.to_string())
    }
}

/// SMTP result type
pub type SmtpResult = Result<Response, SmtpError>;

#[cfg(test)]
mod test {
    // TODO
}
