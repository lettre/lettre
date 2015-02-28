// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! ESMTP features

use std::str::FromStr;
use std::fmt::{Display, Formatter, Result};
use std::result::Result as RResult;

use common::CRLF;
use response::Response;
use self::Extension::{EightBitMime, SmtpUtfEight, StartTls, Size};

/// Supported ESMTP keywords
#[derive(PartialEq,Eq,Copy,Clone,Debug)]
pub enum Extension {
    /// 8BITMIME keyword
    ///
    /// RFC 6152 : https://tools.ietf.org/html/rfc6152
    EightBitMime,
    /// SMTPUTF8 keyword
    ///
    /// RFC 6531 : https://tools.ietf.org/html/rfc6531
    SmtpUtfEight,
    /// STARTTLS keyword
    ///
    /// RFC 2487 : http://tools.ietf.org/html/rfc2487
    StartTls,
    /// SIZE keyword
    ///
    /// RFC 1427 : https://tools.ietf.org/html/rfc1427
    Size(usize),
}

impl Display for Extension {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write! (f, "{}",
            match self {
                &EightBitMime => "8BITMIME".to_string(),
                &SmtpUtfEight => "SMTPUTF8".to_string(),
                &StartTls => "STARTTLS".to_string(),
                &Size(ref size) => format!("SIZE={}", size),
            }
        )
    }
}

impl FromStr for Extension {
    // TODO: check RFC
    type Err = &'static str;
    fn from_str(s: &str) -> RResult<Extension, &'static str> {
        let splitted : Vec<&str> = s.splitn(1, ' ').collect();
        match splitted.len() {
            1 => match splitted[0] {
                     "8BITMIME" => Ok(EightBitMime),
                     "SMTPUTF8" => Ok(SmtpUtfEight),
                     "STARTTLS" => Ok(StartTls),
                     _ => Err("Unknown extension"),
                 },
            2 => match (splitted[0], splitted[1].parse::<usize>()) {
                     ("SIZE", Ok(size)) => Ok(Size(size)),
                     _ => Err("Can't parse size"),
                 },
            _ => Err("Empty extension?"),
        }
    }
}

impl Extension {
    /// Checks if the ESMTP keyword is the same
    pub fn same_extension_as(&self, other: &Extension) -> bool {
        if self == other {
            return true;
        }
        match (self, other) {
            (&Size(_), &Size(_)) => true,
            _ => false,
        }
    }

    /// Parses supported ESMTP features
    pub fn parse_esmtp_response(message: &str) -> Vec<Extension> {
        let mut esmtp_features = Vec::new();
        for line in message.split(CRLF) {
            if let Ok(Response{code: 250, message}) = line.parse::<Response>() {
                if let Ok(keyword) = message.unwrap().as_slice().parse::<Extension>() {
                    esmtp_features.push(keyword);
                };
            }
        }
        esmtp_features
    }

    /// Returns the string to add to the mail command
    pub fn client_mail_option(&self) -> Option<&str> {
        match *self {
            EightBitMime => Some("BODY=8BITMIME"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Extension;

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", Extension::EightBitMime), "8BITMIME".to_string());
        assert_eq!(format!("{}", Extension::Size(42)), "SIZE=42".to_string());
    }

    #[test]
    fn test_from_str() {
        assert_eq!("8BITMIME".parse::<Extension>(), Ok(Extension::EightBitMime));
        assert_eq!("SIZE 42".parse::<Extension>(), Ok(Extension::Size(42)));
        assert!("SIZ 42".parse::<Extension>().is_err());
        assert!("SIZE 4a2".parse::<Extension>().is_err());
        // TODO: accept trailing spaces ?
        assert!("SIZE 42 ".parse::<Extension>().is_err());
    }

    #[test]
    fn test_same_extension_as() {
        assert_eq!(Extension::EightBitMime.same_extension_as(&Extension::EightBitMime), true);
        assert_eq!(Extension::Size(42).same_extension_as(&Extension::Size(42)), true);
        assert_eq!(Extension::Size(42).same_extension_as(&Extension::Size(43)), true);
        assert_eq!(Extension::Size(42).same_extension_as(&Extension::EightBitMime), false);
        assert_eq!(Extension::EightBitMime.same_extension_as(&Extension::SmtpUtfEight), false);
    }

    #[test]
    fn test_parse_esmtp_response() {
        assert_eq!(Extension::parse_esmtp_response("me\r\n250-8BITMIME\r\n250 SIZE 42"),
            vec!(Extension::EightBitMime, Extension::Size(42)));
        assert_eq!(Extension::parse_esmtp_response("me\r\n250-8BITMIME\r\n250 UNKNON 42"),
            vec!(Extension::EightBitMime));
        assert_eq!(Extension::parse_esmtp_response("me\r\n250-9BITMIME\r\n250 SIZE a"),
            vec!());
        assert_eq!(Extension::parse_esmtp_response("me\r\n250-SIZE 42\r\n250 SIZE 43"),
            vec!(Extension::Size(42), Extension::Size(43)));
        assert_eq!(Extension::parse_esmtp_response(""),
            vec!());
    }
}
