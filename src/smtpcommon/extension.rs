// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP commands and ESMTP features library

use std::from_str::FromStr;
use std::fmt::{Show, Formatter, Result};

/// Supported ESMTP keywords
#[deriving(PartialEq,Eq,Clone)]
pub enum SmtpExtension {
    /// 8BITMIME keyword
    ///
    /// RFC 6152 : https://tools.ietf.org/html/rfc6152
    EightBitMime,
    /// SMTPUTF8 keyword
    ///
    /// RFC 6531 : https://tools.ietf.org/html/rfc6531
    SmtpUtfEight,
    /// SIZE keyword
    ///
    /// RFC 1427 : https://tools.ietf.org/html/rfc1427
    Size(uint)
}

impl Show for SmtpExtension {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write(
            match self {
                &EightBitMime   => "8BITMIME".to_string(),
                &SmtpUtfEight   => "SMTPUTF8".to_string(),
                &Size(ref size) => format!("SIZE={}", size)
            }.as_bytes()
        )
    }
}

impl FromStr for SmtpExtension {
    fn from_str(s: &str) -> Option<SmtpExtension> {
        let splitted : Vec<&str> = s.splitn(1, ' ').collect();
        match splitted.len() {
            1 => match splitted[0] {
                     "8BITMIME" => Some(EightBitMime),
                     "SMTPUTF8" => Some(SmtpUtfEight),
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

impl SmtpExtension {
    /// Checks if the ESMTP keyword is the same
    pub fn same_extension_as(&self, other: SmtpExtension) -> bool {
        if *self == other {
            return true;
        }
        match (*self, other) {
            (Size(_), Size(_)) => true,
            _                  => false
        }
    }
}

#[cfg(test)]
mod test {
    use smtpcommon::extension;
    use smtpcommon::extension::SmtpExtension;

    #[test]
    fn test_extension_same_extension_as() {
        assert_eq!(extension::EightBitMime.same_extension_as(extension::EightBitMime), true);
        assert_eq!(extension::Size(42).same_extension_as(extension::Size(42)), true);
        assert_eq!(extension::Size(42).same_extension_as(extension::Size(43)), true);
        assert_eq!(extension::Size(42).same_extension_as(extension::EightBitMime), false);
    }

    #[test]
    fn test_extension_fmt() {
        assert_eq!(format!("{}", extension::EightBitMime), "8BITMIME".to_string());
        assert_eq!(format!("{}", extension::Size(42)), "SIZE=42".to_string());
    }

    #[test]
    fn test_extension_from_str() {
        assert_eq!(from_str::<SmtpExtension>("8BITMIME"), Some(extension::EightBitMime));
        assert_eq!(from_str::<SmtpExtension>("SIZE 42"), Some(extension::Size(42)));
        assert_eq!(from_str::<SmtpExtension>("SIZ 42"), None);
        assert_eq!(from_str::<SmtpExtension>("SIZE 4a2"), None);
        // TODO: accept trailing spaces ?
        assert_eq!(from_str::<SmtpExtension>("SIZE 42 "), None);
    }
}
