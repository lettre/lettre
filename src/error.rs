// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Error and result type for SMTP clients

#![unstable]

use std::error::Error;
use std::io::IoError;
use std::error::FromError;

use response::Response;
use self::ErrorKind::{TransientError, PermanentError, UnknownError, InternalIoError};

/// An enum of all error kinds.
#[derive(PartialEq, Eq, Clone, Show)]
pub enum ErrorKind {
    /// Transient error
    ///
    /// 4xx reply code
    TransientError(Response),
    /// permanent error
    ///
    /// 5xx reply code
    PermanentError(Response),
    /// Unknown error
    UnknownError(String),
    /// IO error
    InternalIoError(IoError),
}

/// smtp error type
#[derive(PartialEq, Eq, Clone, Show)]
pub struct SmtpError {
    /// Error kind
    pub kind: ErrorKind,
    /// Error description
    pub desc: &'static str,
    /// Error cause
    pub detail: Option<String>,
}

impl FromError<IoError> for SmtpError {
    fn from_error(err: IoError) -> SmtpError {
        SmtpError {
            kind: InternalIoError(err),
            desc: "An internal IO error ocurred.",
            detail: None,
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
            4 => TransientError(response),
            5 => PermanentError(response),
            _ => UnknownError(format! ("{:?}", response)),
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
            detail: None,
        }
    }
}

impl FromError<&'static str> for SmtpError {
    fn from_error(string: &'static str) -> SmtpError {
        SmtpError {
            kind: UnknownError(string.to_string()),
            desc: "an unknown error occured during the SMTP transaction",
            detail: None,
        }
    }
}

impl Error for SmtpError {
    fn description(&self) -> &str {
        match self.kind {
            InternalIoError(ref err) => err.desc,
            _ => self.desc,
        }
    }

    fn detail(&self) -> Option<String> {
        match self.kind {
            TransientError(ref response) => Some(format! ("{:?}", response)),
            PermanentError(ref response) => Some(format! ("{:?}", response)),
            UnknownError(ref string) => Some(string.to_string()),
            InternalIoError(ref err) => err.detail.clone(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self.kind {
            InternalIoError(ref err) => Some(&*err as &Error),
            _ => None,
        }
    }
}

/// SMTP result type
pub type SmtpResult = Result<Response, SmtpError>;
