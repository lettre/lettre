// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP command

#![unstable]

use std::error::FromError;
use std::fmt::{Show, Formatter, Result};

use response::Response;
use error::SmtpResult;
use common::SP;

/// Supported SMTP commands
///
/// We do not implement the following SMTP commands, as they were deprecated in RFC 5321
/// and must not be used by clients:
/// `SEND`, `SOML`, `SAML`, `TURN`
#[deriving(PartialEq,Eq,Clone)]
pub enum Command {
    /// A fake command to represent the connection step
    Connect,
    /// Start a TLS tunnel
    StartTls,
    /// Extended Hello command
    ExtendedHello(String),
    /// Hello command
    Hello(String),
    /// Mail command, takes optional options
    Mail(String, Option<Vec<String>>),
    /// Recipient command, takes optional options
    Recipient(String, Option<Vec<String>>),
    /// Data command
    Data,
    /// A fake command to represent the message content
    Message,
    /// Reset command
    Reset,
    /// Verify command, takes optional options
    Verify(String),
    /// Expand command, takes optional options
    Expand(String),
    /// Help command, takes optional options
    Help(Option<String>),
    /// Noop command
    Noop,
    /// Quit command
    Quit,
}

impl Show for Command {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write( match *self {
            Connect => "CONNECT".to_string(),
            StartTls => "STARTTLS".to_string(),
            ExtendedHello(ref my_hostname) => format!("EHLO {}", my_hostname),
            Hello(ref my_hostname) => format!("HELO {}", my_hostname),
            Mail(ref from_address, None) => format!("MAIL FROM:<{}>", from_address),
            Mail(ref from_address, Some(ref options)) =>
                format!("MAIL FROM:<{}> {}", from_address, options.connect(SP)),
            Recipient(ref to_address, None) => format!("RCPT TO:<{}>", to_address),
            Recipient(ref to_address, Some(ref options)) =>
                format!("RCPT TO:<{}> {}", to_address, options.connect(SP)),
            Data => "DATA".to_string(),
            Message => "MESSAGE".to_string(),
            Reset => "RSET".to_string(),
            Verify(ref address) => format!("VRFY {}", address),
            Expand(ref address) => format!("EXPN {}", address),
            Help(None) => "HELP".to_string(),
            Help(Some(ref argument)) => format!("HELP {}", argument),
            Noop => "NOOP".to_string(),
            Quit => "QUIT".to_string(),
        }.as_bytes())
    }
}

impl Command {
    /// Tests if the `Command` is ASCII-only
    pub fn is_ascii(&self) -> bool {
        match *self {
            ExtendedHello(ref my_hostname) => my_hostname.is_ascii(),
            Hello(ref my_hostname) => my_hostname.is_ascii(),
            Mail(ref from_address, None) => from_address.is_ascii(),
            Mail(ref from_address, Some(ref options)) => from_address.is_ascii()
                                                         && options.concat().is_ascii(),
            Recipient(ref to_address, None) => to_address.is_ascii(),
            Recipient(ref to_address, Some(ref options)) => to_address.is_ascii()
                                                            && options.concat().is_ascii(),
            Verify(ref address) => address.is_ascii(),
            Expand(ref address) => address.is_ascii(),
            Help(Some(ref argument)) => argument.is_ascii(),
            _ => true
        }
    }

    /// Tests if the command was successful
    ///
    /// Returns `Ok` if the given response is considered successful for the `Command`,
    /// `Err` otherwise
    pub fn test_success(&self, response: Response) -> SmtpResult {
        let success = match *self {
            Connect => vec![220],
            StartTls => vec![220],
            ExtendedHello(..) => vec![250],
            Hello(..) => vec![250],
            Mail(..) => vec![250],
            Recipient(..) => vec![250, 251],
            Data => vec![354],
            Message => vec![250],
            Reset => vec![250],
            Verify(..) => vec![250, 251, 252],
            Expand(..) => vec![250, 252],
            Help(..) => vec![211, 214],
            Noop => vec![250],
            Quit => vec![221],
        }.contains(&response.code);
        if success {
            Ok(response)
        } else {
            Err(FromError::from_error(response))
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_fmt() {
        assert_eq!(
            format!("{}", super::Noop),
            "NOOP".to_string()
        );
        assert_eq!(
            format!("{}", super::ExtendedHello("my_name".to_string())),
            "EHLO my_name".to_string()
        );
        assert_eq!(
            format!("{}", super::Mail("test".to_string(), Some(vec!["option".to_string()]))),
            "MAIL FROM:<test> option".to_string()
        );
        assert_eq!(
            format!("{}", super::Mail("test".to_string(),
                          Some(vec!["option".to_string(), "option2".to_string()]))),
            "MAIL FROM:<test> option option2".to_string()
        );
    }

    #[test]
    fn test_is_ascii() {
        assert!(super::Help(None).is_ascii());
        assert!(super::ExtendedHello("my_name".to_string()).is_ascii());
        assert!(!super::ExtendedHello("my_namé".to_string()).is_ascii());
        assert!(
            super::Mail("test".to_string(), Some(vec!["option".to_string(), "option2".to_string()]))
        .is_ascii());
        assert!(
            !super::Mail("test".to_string(), Some(vec!["option".to_string(), "option2à".to_string()]))
        .is_ascii());
    }
}
