// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP commands and ESMTP features library

use std::fmt::{Show, Formatter, Result};
use std::io::net::ip::Port;
use std::from_str::FromStr;

/// Default SMTP port
pub static SMTP_PORT: Port = 25;
//pub static SMTPS_PORT: Port = 465;
//pub static SUBMISSION_PORT: Port = 587;

/// Supported SMTP commands
///
/// We do not implement the following SMTP commands, as they were deprecated in RFC 5321
/// and must not be used by clients:
/// SEND, SOML, SAML, TURN
#[deriving(Eq,Clone)]
pub enum SmtpCommand<T> {
    /// Extended Hello command
    ExtendedHello(T),
    /// Hello command
    Hello(T),
    /// Mail command, takes optionnal options
    Mail(T, Option<Vec<T>>),
    /// Recipient command, takes optionnal options
    Recipient(T, Option<Vec<T>>),
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

impl<T: Show + Str> Show for SmtpCommand<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.buf.write(match *self {
            ExtendedHello(ref my_hostname) =>
                format!("EHLO {}", my_hostname.clone()),
            Hello(ref my_hostname) =>
                format!("HELO {}", my_hostname.clone()),
            Mail(ref from_address, None) =>
                format!("MAIL FROM:<{}>", from_address.clone()),
            Mail(ref from_address, Some(ref options)) =>
                format!("MAIL FROM:<{}> {}", from_address.clone(), options.connect(" ")),
            Recipient(ref to_address, None) =>
                format!("RCPT TO:<{}>", to_address.clone()),
            Recipient(ref to_address, Some(ref options)) =>
                format!("RCPT TO:<{}> {}", to_address.clone(), options.connect(" ")),
            Data => "DATA".to_owned(),
            Reset => "RSET".to_owned(),
            Verify(ref address) =>
                format!("VRFY {}", address.clone()),
            Expand(ref address) =>
                format!("EXPN {}", address.clone()),
            Help(None) => "HELP".to_owned(),
            Help(Some(ref argument)) =>
                format!("HELP {}", argument.clone()),
            Noop => "NOOP".to_owned(),
            Quit => "QUIT".to_owned(),
        }.as_bytes())
    }
}

/// Supported ESMTP keywords
#[deriving(Eq,Clone)]
pub enum EsmtpParameter {
    /// 8BITMIME keyword
    ///
    /// RFC 6152 : https://tools.ietf.org/html/rfc6152
    EightBitMime,
    /// SIZE keyword
    ///
    /// RFC 1427 : https://tools.ietf.org/html/rfc1427
    Size(uint)
}

impl Show for EsmtpParameter {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.buf.write(
            match self {
                &EightBitMime  => "8BITMIME".to_owned(),
                &Size(ref size) => format!("SIZE={}", size)
            }.as_bytes()
        )
    }
}

impl FromStr for EsmtpParameter {
    fn from_str(s: &str) -> Option<EsmtpParameter> {
        let splitted : ~[&str] = s.splitn(' ', 1).collect();
        match splitted.len() {
            1 => match splitted[0] {
                     "8BITMIME" => Some(EightBitMime),
                     _          => None
                 },
            2 => match (splitted[0], from_str::<uint>(splitted[1])) {
                     ("SIZE", Some(size)) => Some(Size(size)),
                     _                    => None
                 },
            _          => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{SmtpCommand, EsmtpParameter};

    #[test]
    fn test_command_fmt() {
        let noop: SmtpCommand<StrBuf> = super::Noop;
        assert!(format!("{}", noop) == "NOOP".to_owned());
        assert!(format!("{}", super::ExtendedHello("me")) == "EHLO me".to_owned());
        assert!(format!("{}", 
            super::Mail("test", Some(vec!("option")))) == "MAIL FROM:<test> option".to_owned()
        );
    }

    #[test]
    fn test_esmtp_parameter_fmt() {
        assert!(format!("{}", super::EightBitMime) == "8BITMIME".to_owned());
        assert!(format!("{}", super::Size(42)) == "SIZE=42".to_owned());
    }

    #[test]
    fn test_esmtp_parameter_from_str() {
        assert!(from_str::<EsmtpParameter>("8BITMIME") == Some(super::EightBitMime));
        assert!(from_str::<EsmtpParameter>("SIZE 42") == Some(super::Size(42)));
        assert!(from_str::<EsmtpParameter>("SIZ 42") == None);
        assert!(from_str::<EsmtpParameter>("SIZE 4a2") == None);
        // TODO: accept trailing spaces ?
        assert!(from_str::<EsmtpParameter>("SIZE 42 ") == None);
    }
}
