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

use response::Response;
use self::Extension::*;

/// Supported ESMTP keywords
#[derive(PartialEq,Eq,Copy,Clone,Debug)]
pub enum Extension {
    /// 8BITMIME keyword
    ///
    /// RFC 6152: https://tools.ietf.org/html/rfc6152
    EightBitMime,
    /// SMTPUTF8 keyword
    ///
    /// RFC 6531: https://tools.ietf.org/html/rfc6531
    SmtpUtfEight,
    /// STARTTLS keyword
    ///
    /// RFC 2487: https://tools.ietf.org/html/rfc2487
    StartTls,
    /// AUTH PLAIN mecanism
    ///
    /// RFC 4616: https://tools.ietf.org/html/rfc4616
    PlainAuthentication,
    /// AUTH CRAM-MD5 mecanism
    ///
    /// RFC 2195: https://tools.ietf.org/html/rfc2195
    CramMd5Authentication,
}

impl Extension {
    fn from_str(s: &str) -> Result<Vec<Extension>, &'static str> {
        let splitted : Vec<&str> = s.split(' ').collect();
        match (splitted[0], splitted.len()) {
            ("8BITMIME", 1) => Ok(vec![EightBitMime]),
            ("SMTPUTF8", 1) => Ok(vec![SmtpUtfEight]),
            ("STARTTLS", 1) => Ok(vec![StartTls]),
            ("AUTH", _) => {
                let mut mecanisms: Vec<Extension> = vec![];
                for &mecanism in &splitted[1..] {
                    match mecanism {
                        "PLAIN" => mecanisms.push(PlainAuthentication),
                        "CRAM-MD5" => mecanisms.push(CramMd5Authentication),
                        _ => (),
                    }
                }
                Ok(mecanisms)
            },
            _ => Err("Unknown extension"),
        }
    }

    /// Parses supported ESMTP features
    pub fn parse_esmtp_response(response: &Response) -> Vec<Extension> {
        let mut esmtp_features: Vec<Extension> = Vec::new();

        for line in response.message() {
            if let Ok(keywords) = Extension::from_str(&line) {
                for keyword in keywords {
                    esmtp_features.push(keyword);
                }
            };
        }

        esmtp_features
    }
}

#[cfg(test)]
mod test {
    use super::Extension;
    use response::{Severity, Category, Response, Code};

    #[test]
    fn test_from_str() {
        assert_eq!(Extension::from_str("8BITMIME"), Ok(vec![Extension::EightBitMime]));
        assert_eq!(Extension::from_str("AUTH PLAIN"), Ok(vec![Extension::PlainAuthentication]));
        assert_eq!(Extension::from_str("AUTH PLAIN LOGIN CRAM-MD5"), Ok(vec![Extension::PlainAuthentication, Extension::CramMd5Authentication]));
        assert_eq!(Extension::from_str("AUTH CRAM-MD5 PLAIN"), Ok(vec![Extension::CramMd5Authentication, Extension::PlainAuthentication]));
        assert_eq!(Extension::from_str("AUTH DIGEST-MD5 PLAIN CRAM-MD5"), Ok(vec![Extension::PlainAuthentication, Extension::CramMd5Authentication]));
    }

    #[test]
    fn test_parse_esmtp_response() {
        assert_eq!(Extension::parse_esmtp_response(&Response::new(
            Code::new(
                "2".parse::<Severity>().unwrap(),
                "2".parse::<Category>().unwrap(),
                1,
            ),
            vec!["me".to_string(), "8BITMIME".to_string(), "SIZE 42".to_string()]
        )), vec![Extension::EightBitMime]);
        assert_eq!(Extension::parse_esmtp_response(&Response::new(
            Code::new(
                "4".parse::<Severity>().unwrap(),
                "3".parse::<Category>().unwrap(),
                3,
            ),
            vec!["me".to_string(), "8BITMIME".to_string(), "AUTH PLAIN CRAM-MD5".to_string()]
        )), vec![Extension::EightBitMime, Extension::PlainAuthentication, Extension::CramMd5Authentication]);
    }
}
