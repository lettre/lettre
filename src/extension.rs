// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! ESMTP features library

#![unstable]

use std::from_str::FromStr;
use std::fmt::{Show, Formatter, Result};

use common::CRLF;
use response::Response;

/// Supported ESMTP keywords
#[deriving(PartialEq,Eq,Clone)]
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
    Size(uint)
}

impl Show for Extension {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write(
            match self {
                &EightBitMime   => "8BITMIME".to_string(),
                &SmtpUtfEight   => "SMTPUTF8".to_string(),
                &StartTls       => "STARTTLS".to_string(),
                &Size(ref size) => format!("SIZE={}", size)
            }.as_bytes()
        )
    }
}

impl FromStr for Extension {
    // TODO: check RFC
    fn from_str(s: &str) -> Option<Extension> {
        let splitted : Vec<&str> = s.splitn(1, ' ').collect();
        match splitted.len() {
            1 => match splitted[0] {
                     "8BITMIME" => Some(EightBitMime),
                     "SMTPUTF8" => Some(SmtpUtfEight),
                     "STARTTLS" => Some(StartTls),
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

impl Extension {
    /// Checks if the ESMTP keyword is the same
    pub fn same_extension_as(&self, other: Extension) -> bool {
        if *self == other {
            return true;
        }
        match (*self, other) {
            (Size(_), Size(_)) => true,
            _                  => false
        }
    }

    /// Parses supported ESMTP features
    pub fn parse_esmtp_response(message: &str) -> Option<Vec<Extension>> {
        let mut esmtp_features = Vec::new();
        for line in message.split_str(CRLF) {
            match from_str::<Response>(line) {
                Some(Response{code: 250, message}) => {
                    match from_str::<Extension>(message.unwrap().as_slice()) {
                        Some(keyword) => esmtp_features.push(keyword),
                        None          => ()
                    }
                },
                _ => ()
            }
        }
        Some(esmtp_features)
    }

    /// Returns the string to add to the mail command
    pub fn client_mail_option(&self) -> Option<&str> {
        match *self {
            EightBitMime => Some("BODY=8BITMIME"),
            _ => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::Extension;

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", super::EightBitMime), "8BITMIME".to_string());
        assert_eq!(format!("{}", super::Size(42)), "SIZE=42".to_string());
    }

    #[test]
    fn test_from_str() {
        assert_eq!(from_str::<Extension>("8BITMIME"), Some(super::EightBitMime));
        assert_eq!(from_str::<Extension>("SIZE 42"), Some(super::Size(42)));
        assert_eq!(from_str::<Extension>("SIZ 42"), None);
        assert_eq!(from_str::<Extension>("SIZE 4a2"), None);
        // TODO: accept trailing spaces ?
        assert_eq!(from_str::<Extension>("SIZE 42 "), None);
    }

    #[test]
    fn test_same_extension_as() {
        assert_eq!(super::EightBitMime.same_extension_as(super::EightBitMime), true);
        assert_eq!(super::Size(42).same_extension_as(super::Size(42)), true);
        assert_eq!(super::Size(42).same_extension_as(super::Size(43)), true);
        assert_eq!(super::Size(42).same_extension_as(super::EightBitMime), false);
    }

    #[test]
    fn test_parse_esmtp_response() {
        assert_eq!(Extension::parse_esmtp_response("me\r\n250-8BITMIME\r\n250 SIZE 42"),
            Some(vec![super::EightBitMime, super::Size(42)]));
        assert_eq!(Extension::parse_esmtp_response("me\r\n250-8BITMIME\r\n250 UNKNON 42"),
            Some(vec![super::EightBitMime]));
        assert_eq!(Extension::parse_esmtp_response("me\r\n250-9BITMIME\r\n250 SIZE a"),
            Some(vec![]));
        assert_eq!(Extension::parse_esmtp_response("me\r\n250-SIZE 42\r\n250 SIZE 43"),
            Some(vec![super::Size(42), super::Size(43)]));
        assert_eq!(Extension::parse_esmtp_response(""),
            Some(vec![]));
    }
}
