// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! TODO

use std::error::Error;
use std::io::IoError;
use std::error::FromError;

use response::Response;

/// An enum of all error kinds.
#[deriving(PartialEq, Eq, Clone, Show)]
pub enum ErrorKind {
    /// TODO
    TransientError(Response),
    /// TODO
    PermanentError(Response),
    /// TODO
    UnknownError(String),
    /// TODO
    InternalIoError(IoError),
}

/// TODO
#[deriving(PartialEq, Eq, Clone, Show)]
pub struct SmtpError {
    /// TODO
    pub kind: ErrorKind,
    /// TODO
    pub desc: &'static str,
    /// TODO
    pub detail: Option<String>,
}


impl FromError<IoError> for SmtpError {
    fn from_error(err: IoError) -> SmtpError {
        SmtpError {
            kind: InternalIoError(err),
            desc: "An internal IO error ocurred.",
            detail: None
        }
    }
}

impl FromError<(ErrorKind, &'static str)> for SmtpError {
    fn from_error((kind, desc): (ErrorKind, &'static str)) -> SmtpError {
        SmtpError {
            kind: kind,
            desc: desc,
            detail: None,
        }
    }
}

impl FromError<Response> for SmtpError {
    fn from_error(response: Response) -> SmtpError {
        let kind = match response.code/100 {
            4 => TransientError(response.clone()),
            5 => PermanentError(response.clone()),
            _ => UnknownError(response.clone().to_string()),
        };
        let desc = match kind {
            TransientError(_) => "a permanent error occured during the SMTP transaction",
            PermanentError(_) => "a permanent error occured during the SMTP transaction",
            UnknownError(_) => "an unknown error occured during the SMTP transaction",
            InternalIoError(_) => "an I/O error occurred",
        };
        SmtpError {
            kind: kind,
            desc: desc,
            detail: None
        }
    }
}

impl FromError<&'static str> for SmtpError {
    fn from_error(string: &'static str) -> SmtpError {
        SmtpError {
            kind: UnknownError(string.to_string()),
            desc: "an unknown error occured during the SMTP transaction",
            detail: None
        }
    }
}

impl Error for SmtpError {
    fn description(&self) -> &str {
        match self.kind {
            TransientError(_) => "a permanent error occured during the SMTP transaction",
            PermanentError(_) => "a permanent error occured during the SMTP transaction",
            UnknownError(_) => "an unknown error occured during the SMTP transaction",
            InternalIoError(_) => "an I/O error occurred",
        }
    }

    fn detail(&self) -> Option<String> {
        match self.kind {
            TransientError(ref response) => Some(response.to_string()),
            PermanentError(ref response) => Some(response.to_string()),
            UnknownError(ref string) => Some(string.to_string()),
            _ => None,
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self.kind {
            InternalIoError(ref err) => Some(&*err as &Error),
            _ => None,
        }
    }
}

/// Library generic result type
pub type SmtpResult<T> = Result<T, SmtpError>;
