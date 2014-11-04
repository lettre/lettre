// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![unstable]

//! Represents a valid complete SMTP command, ready to be sent to a server

use std::fmt::{Show, Formatter, Result};
use common::SP;

/// Supported SMTP commands
///
/// We do not implement the following SMTP commands, as they were deprecated in RFC 5321
/// and must not be used by clients:
/// SEND, SOML, SAML, TURN
#[deriving(PartialEq,Eq,Clone)]
pub enum Command {
    /// A fake command to represent the connection step
    Connect,
    /// Extended Hello command
    ExtendedHello(String),
    /// Hello command
    Hello(String),
    /// Mail command, takes optionnal options
    Mail(String, Option<Vec<String>>),
    /// Recipient command, takes optionnal options
    Recipient(String, Option<Vec<String>>),
    /// Data command
    Data,
    /// Reset command
    Reset,
    /// Verify command, takes optionnal options
    Verify(String, Option<Vec<String>>),
    /// Expand command, takes optionnal options
    Expand(String, Option<Vec<String>>),
    /// Help command, takes optionnal options
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
            ExtendedHello(ref my_hostname) =>
                format!("EHLO {}", my_hostname.clone()),
            Hello(ref my_hostname) =>
                format!("HELO {}", my_hostname.clone()),
            Mail(ref from_address, None) =>
                format!("MAIL FROM:<{}>", from_address.clone()),
            Mail(ref from_address, Some(ref options)) =>
                format!("MAIL FROM:<{}> {}", from_address.clone(), options.connect(SP)),
            Recipient(ref to_address, None) =>
                format!("RCPT TO:<{}>", to_address.clone()),
            Recipient(ref to_address, Some(ref options)) =>
                format!("RCPT TO:<{}> {}", to_address.clone(), options.connect(SP)),
            Data => "DATA".to_string(),
            Reset => "RSET".to_string(),
            Verify(ref address, None) =>
                format!("VRFY {}", address.clone()),
            Verify(ref address, Some(ref options)) =>
                format!("VRFY {} {}", address.clone(), options.connect(SP)),
            Expand(ref address, None) =>
                format!("EXPN {}", address.clone()),
            Expand(ref address, Some(ref options)) =>
                format!("EXPN {} {}", address.clone(), options.connect(SP)),
            Help(None) => "HELP".to_string(),
            Help(Some(ref argument)) =>
                format!("HELP {}", argument.clone()),
            Noop => "NOOP".to_string(),
            Quit => "QUIT".to_string(),
        }.as_bytes())
    }
}

impl Command {
    /// Tests if the Command is ASCII-only
    pub fn is_ascii(&self) -> bool {
        match *self {
            ExtendedHello(ref my_hostname) => my_hostname.is_ascii(),
            Hello(ref my_hostname) => my_hostname.is_ascii(),
            Mail(ref from_address, None) => from_address.is_ascii(),
            Mail(ref from_address, Some(ref options)) => from_address.is_ascii() && options.concat().is_ascii(),
            Recipient(ref to_address, None) => to_address.is_ascii(),
            Recipient(ref to_address, Some(ref options)) => to_address.is_ascii() && options.concat().is_ascii(),
            Verify(ref address, None) => address.is_ascii(),
            Verify(ref address, Some(ref options)) => address.is_ascii() && options.concat().is_ascii(),
            Expand(ref address, None) => address.is_ascii(),
            Expand(ref address, Some(ref options)) => address.is_ascii() && options.concat().is_ascii(),
            Help(Some(ref argument)) => argument.is_ascii(),
            _ => true
        }
    }
}

#[cfg(test)]
mod test {
    use command;

    #[test]
    fn test_fmt() {
        assert_eq!(
            format!("{}", command::Noop),
            format!("NOOP")
        );
        assert_eq!(
            format!("{}", command::ExtendedHello("my_name".to_string())),
            format!("EHLO my_name")
        );
        assert_eq!(
            format!("{}", command::Mail("test".to_string(), Some(vec!("option".to_string())))),
            format!("MAIL FROM:<test> option")
        );
        assert_eq!(
            format!("{}", command::Mail("test".to_string(), Some(vec!("option".to_string(), "option2".to_string())))),
            format!("MAIL FROM:<test> option option2")
        );
    }

    #[test]
    fn test_is_ascii() {
        assert!(command::Help(None).is_ascii());
        assert!(command::ExtendedHello("my_name".to_string()).is_ascii());
        assert!(!command::ExtendedHello("my_namé".to_string()).is_ascii());
        assert!(command::Mail("test".to_string(), Some(vec!("option".to_string(), "option2".to_string()))).is_ascii());
        assert!(!command::Mail("test".to_string(), Some(vec!("option".to_string(), "option2à".to_string()))).is_ascii());
    }
}
