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
use std::result::Result;

use common::CRLF;
use response::Response;
use self::Extension::{PlainAuthentication, CramMd5Authentication, EightBitMime, SmtpUtfEight, StartTls, Size};

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
    /// AUTH PLAIN
    PlainAuthentication,
    /// AUTH CRAM-MD5
    CramMd5Authentication,
}

impl Extension {
    // TODO: check RFC
    fn from_str(s: &str) -> Result<Vec<Extension>, &'static str> {
        let splitted : Vec<&str> = s.split(' ').collect();
        match (splitted[0], splitted.len()) {
            ("8BITMIME", 1) => Ok(vec!(EightBitMime)),
            ("SMTPUTF8", 1) => Ok(vec!(SmtpUtfEight)),
            ("STARTTLS", 1) => Ok(vec!(StartTls)),
            ("SIZE", 2) => match splitted[1].parse::<usize>() {
                               Ok(size) => Ok(vec!(Size(size))),
                               _ => Err("Can't parse size"),
                           },
            ("AUTH", _) => {
                let mut mecanisms: Vec<Extension> = vec!();
                for &mecanism in &splitted[1..] {
                    match mecanism {
                        "PLAIN" => mecanisms.push(PlainAuthentication),
                        _ => (),
                    }
                }
                Ok(mecanisms)
            },
            _ => Err("Unknown extension"),
        }
    }

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
        let mut esmtp_features: Vec<Extension> = Vec::new();
        for line in message.split(CRLF) {
            if let Ok(Response{code: 250, message}) = line.parse::<Response>() {
                if let Ok(keywords) = Extension::from_str(message.unwrap().as_slice()) {
                    esmtp_features.push_all(&keywords);
                };
            }
        }
        esmtp_features
    }
}

#[cfg(test)]
mod test {
    use super::Extension;

    #[test]
    fn test_from_str() {
        assert_eq!(Extension::from_str("8BITMIME"), Ok(vec!(Extension::EightBitMime)));
        assert_eq!(Extension::from_str("SIZE 42"), Ok(vec!(Extension::Size(42))));
        assert_eq!(Extension::from_str("AUTH PLAIN"), Ok(vec!(Extension::PlainAuthentication)));
        assert_eq!(Extension::from_str("AUTH PLAIN CRAM-MD5"), Ok(vec!(Extension::PlainAuthentication)));
        assert_eq!(Extension::from_str("AUTH CRAM-MD5 PLAIN"), Ok(vec!(Extension::PlainAuthentication)));
        assert_eq!(Extension::from_str("AUTH DIGEST-MD5 PLAIN CRAM-MD5"), Ok(vec!(Extension::PlainAuthentication)));
        assert!(Extension::from_str("SIZ 42").is_err());
        assert!(Extension::from_str("SIZE 4a2").is_err());
        // TODO: accept trailing spaces ?
        assert!(Extension::from_str("SIZE 42 ").is_err());
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
