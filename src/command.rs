// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP command

use std::ascii::AsciiExt;
use std::error::FromError;
use std::fmt::{Display, Formatter, Result};

use response::Response;
use error::SmtpResult;
use common::SP;

/// Supported SMTP commands
///
/// We do not implement the following SMTP commands, as they were deprecated in RFC 5321
/// and must not be used by clients: `SEND`, `SOML`, `SAML`, `TURN`.
#[derive(PartialEq,Eq,Clone,Debug)]
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

impl Display for Command {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write! (f, "{}", match *self {
            Command::Connect => "CONNECT".to_string(),
            Command::StartTls => "STARTTLS".to_string(),
            Command::ExtendedHello(ref my_hostname) => format!("EHLO {}", my_hostname),
            Command::Hello(ref my_hostname) => format!("HELO {}", my_hostname),
            Command::Mail(ref from_address, None) => format!("MAIL FROM:<{}>", from_address),
            Command::Mail(ref from_address, Some(ref options)) =>
                format!("MAIL FROM:<{}> {}", from_address, options.connect(SP)),
            Command::Recipient(ref to_address, None) => format!("RCPT TO:<{}>", to_address),
            Command::Recipient(ref to_address, Some(ref options)) =>
                format!("RCPT TO:<{}> {}", to_address, options.connect(SP)),
            Command::Data => "DATA".to_string(),
            Command::Message => "MESSAGE".to_string(),
            Command::Reset => "RSET".to_string(),
            Command::Verify(ref address) => format!("VRFY {}", address),
            Command::Expand(ref address) => format!("EXPN {}", address),
            Command::Help(None) => "HELP".to_string(),
            Command::Help(Some(ref argument)) => format!("HELP {}", argument),
            Command::Noop => "NOOP".to_string(),
            Command::Quit => "QUIT".to_string(),
        })
    }
}

impl Command {
    /// Tests if the `Command` is ASCII-only
    pub fn is_ascii(&self) -> bool {
        match *self {
            Command::ExtendedHello(ref my_hostname) => my_hostname.is_ascii(),
            Command::Hello(ref my_hostname) => my_hostname.is_ascii(),
            Command::Mail(ref from_address, None) => from_address.is_ascii(),
            Command::Mail(ref from_address, Some(ref options)) => from_address.is_ascii()
                                                         && options.concat().is_ascii(),
            Command::Recipient(ref to_address, None) => to_address.is_ascii(),
            Command::Recipient(ref to_address, Some(ref options)) => to_address.is_ascii()
                                                            && options.concat().is_ascii(),
            Command::Verify(ref address) => address.is_ascii(),
            Command::Expand(ref address) => address.is_ascii(),
            Command::Help(Some(ref argument)) => argument.is_ascii(),
            _ => true,
        }
    }

    /// Tests if the command was successful
    ///
    /// Returns `Ok` if the given response is considered successful for the `Command`,
    /// `Err` otherwise
    pub fn test_success(&self, response: Response) -> SmtpResult {
        let success = match *self {
            Command::Connect => vec![220],
            Command::StartTls => vec![220],
            Command::ExtendedHello(..) => vec![250],
            Command::Hello(..) => vec![250],
            Command::Mail(..) => vec![250],
            Command::Recipient(..) => vec![250, 251],
            Command::Data => vec![354],
            Command::Message => vec![250],
            Command::Reset => vec![250],
            Command::Verify(..) => vec![250, 251, 252],
            Command::Expand(..) => vec![250, 252],
            Command::Help(..) => vec![211, 214],
            Command::Noop => vec![250],
            Command::Quit => vec![221],
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
    use super::Command;

    #[test]
    fn test_fmt() {
        assert_eq!(
            format!("{}", Command::Noop),
            "NOOP".to_string()
        );
        assert_eq!(
            format!("{}", Command::ExtendedHello("my_name".to_string())),
            "EHLO my_name".to_string()
        );
        assert_eq!(
            format!("{}", Command::Mail("test".to_string(), Some(vec!["option".to_string()]))),
            "MAIL FROM:<test> option".to_string()
        );
        assert_eq!(
            format!("{}", Command::Mail("test".to_string(),
                          Some(vec!["option".to_string(), "option2".to_string()]))),
            "MAIL FROM:<test> option option2".to_string()
        );
    }

    #[test]
    fn test_is_ascii() {
        assert!(Command::Help(None).is_ascii());
        assert!(Command::ExtendedHello("my_name".to_string()).is_ascii());
        assert!(!Command::ExtendedHello("my_namé".to_string()).is_ascii());
        assert!(
            Command::Mail("test".to_string(), Some(vec!["test".to_string(), "test".to_string()]))
        .is_ascii());
        assert!(
            !Command::Mail("test".to_string(), Some(vec!["est".to_string(), "testà".to_string()]))
        .is_ascii());
        assert!(
            !Command::Mail("testé".to_string(), Some(vec!["est".to_string(), "test".to_string()]))
        .is_ascii());
    }
}
