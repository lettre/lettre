// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*! SMTP commands [1] and ESMTP features [2] library

[1] https://tools.ietf.org/html/rfc5321#section-4.1
[2] http://tools.ietf.org/html/rfc1869

*/

use std::fmt::{Show, Formatter, Result};
use std::io::net::ip::Port;
use std::from_str::FromStr;

/// Default SMTP port
pub static SMTP_PORT: Port = 25;
//pub static SMTPS_PORT: Port = 465;
//pub static SUBMISSION_PORT: Port = 587;

/// SMTP commands
/// We do not implement the following SMTP commands, as they were deprecated in RFC 5321
/// and must not be used by clients :
/// SEND, SOML, SAML, TURN
#[deriving(Eq,Clone)]
pub enum SmtpCommand<T> {
    /// Extended Hello command
    ExtendedHello(T),
    /// Hello command
    Hello(T),
    /// Mail command, takes optionnal options
    Mail(T, Option<T>),
    /// Recipient command, takes optionnal options
    Recipient(T, Option<T>),
    /// Data command
    Data,
    /// Reset command
    Reset,
    /// Verify command
    Verify(T),
    /// Expand command
    Expand(T),
    /// Help command, takes optionnal options
    Help(Option<T>),
    /// Noop command
    Noop,
    /// Quit command
    Quit,

}

impl<T: Show> Show for SmtpCommand<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.buf.write(match *self {
            ExtendedHello(ref my_hostname) =>
                format!("EHLO {}", my_hostname.clone()),
            Hello(ref my_hostname) =>
                format!("HELO {}", my_hostname.clone()),
            Mail(ref from_address, None) =>
                format!("MAIL FROM:{}", from_address.clone()),
            Mail(ref from_address, Some(ref options)) =>
                format!("MAIL FROM:{} {}", from_address.clone(), options.clone()),
            Recipient(ref to_address, None) =>
                format!("RCPT TO:{}", to_address.clone()),
            Recipient(ref to_address, Some(ref options)) =>
                format!("RCPT TO:{} {}", to_address.clone(), options.clone()),
            Data => ~"DATA",
            Reset => ~"RSET",
            Verify(ref address) =>
                format!("VRFY {}", address.clone()),
            Expand(ref address) =>
                format!("EXPN {}", address.clone()),
            Help(None) => ~"HELP",
            Help(Some(ref argument)) =>
                format!("HELP {}", argument.clone()),
            Noop => ~"NOOP",
            Quit => ~"QUIT",
        }.as_bytes())
    }
}

/// Supported ESMTP keywords
#[deriving(Eq,Clone)]
pub enum EsmtpParameter {
    /// 8BITMIME keyword
    /// RFC 6152 : https://tools.ietf.org/html/rfc6152
    EightBitMime,
}

impl Show for EsmtpParameter {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.buf.write(
            match self {
                &EightBitMime  => "8BITMIME".as_bytes()
            }
        )
    }
}

impl FromStr for EsmtpParameter {
    fn from_str(s: &str) -> Option<EsmtpParameter> {
        match s.as_slice() {
            "8BITMIME" => Some(EightBitMime),
            _          => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{EsmtpParameter};

    #[test]
    fn test_command_fmt() {
        //assert!(format!("{}", super::Noop) == ~"NOOP");
        assert!(format!("{}", super::ExtendedHello("me")) == ~"EHLO me");
        assert!(format!("{}", super::Mail("test", Some("option"))) == ~"MAIL FROM:test option");
    }

    #[test]
    fn test_esmtp_parameter_fmt() {
        assert!(format!("{}", super::EightBitMime) == ~"8BITMIME");
    }

    #[test]
    fn test_ehlokeyword_from_str() {
        assert!(from_str::<EsmtpParameter>("8BITMIME") == Some(super::EightBitMime));
    }
}
