// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! TODO

use std::fmt;
use std::fmt::{Show, Formatter};

use extension::SmtpExtension;
use response::SmtpResponse;
use common::CRLF;

/// Information about an SMTP server
#[deriving(Clone)]
pub struct SmtpServerInfo {
    /// Server name
    pub name: String,
    /// ESMTP features supported by the server
    pub esmtp_features: Option<Vec<SmtpExtension>>
}

impl Show for SmtpServerInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write(
            format!("{} with {}",
                self.name,
                match self.esmtp_features.clone() {
                    Some(features) => features.to_string(),
                    None => format!("no supported features")
                }
            ).as_bytes()
        )
    }
}

impl SmtpServerInfo {
    /// Parses supported ESMTP features
    ///
    /// TODO: Improve parsing
    pub fn parse_esmtp_response(message: String) -> Option<Vec<SmtpExtension>> {
        let mut esmtp_features = Vec::new();
        for line in message.as_slice().split_str(CRLF) {
            match from_str::<SmtpResponse>(line) {
                Some(SmtpResponse{code: 250, message: message}) => {
                    match from_str::<SmtpExtension>(message.unwrap().as_slice()) {
                        Some(keyword) => esmtp_features.push(keyword),
                        None          => ()
                    }
                },
                _ => ()
            }
        }
        match esmtp_features.len() {
            0 => None,
            _ => Some(esmtp_features)
        }
    }

    /// Checks if the server supports an ESMTP feature
    pub fn supports_feature(&self, keyword: SmtpExtension) -> Result<SmtpExtension, ()> {
        match self.esmtp_features.clone() {
            Some(esmtp_features) => {
                for feature in esmtp_features.iter() {
                    if keyword.same_extension_as(*feature) {
                        return Ok(*feature);
                    }
                }
                Err({})
            },
            None => Err({})
        }
    }
}

#[cfg(test)]
mod test {
    use super::SmtpServerInfo;
    use extension;

    #[test]
    fn test_smtp_server_info_fmt() {
        assert_eq!(format!("{}", SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::EightBitMime))
        }), "name with [8BITMIME]".to_string());
        assert_eq!(format!("{}", SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::EightBitMime, extension::Size(42)))
        }), "name with [8BITMIME, SIZE=42]".to_string());
        assert_eq!(format!("{}", SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: None
        }), "name with no supported features".to_string());
    }

    #[test]
    fn test_smtp_server_info_parse_esmtp_response() {
        assert_eq!(SmtpServerInfo::parse_esmtp_response(String::from_str("me\r\n250-8BITMIME\r\n250 SIZE 42")),
            Some(vec!(extension::EightBitMime, extension::Size(42))));
        assert_eq!(SmtpServerInfo::parse_esmtp_response(String::from_str("me\r\n250-8BITMIME\r\n250 UNKNON 42")),
            Some(vec!(extension::EightBitMime)));
        assert_eq!(SmtpServerInfo::parse_esmtp_response(String::from_str("me\r\n250-9BITMIME\r\n250 SIZE a")),
            None);
        assert_eq!(SmtpServerInfo::parse_esmtp_response(String::from_str("me\r\n250-SIZE 42\r\n250 SIZE 43")),
            Some(vec!(extension::Size(42), extension::Size(43))));
        assert_eq!(SmtpServerInfo::parse_esmtp_response(String::from_str("")),
            None);
    }

    #[test]
    fn test_smtp_server_info_supports_feature() {
        assert_eq!(SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::EightBitMime))
        }.supports_feature(extension::EightBitMime), Ok(extension::EightBitMime));
        assert_eq!(SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::Size(42), extension::EightBitMime))
        }.supports_feature(extension::EightBitMime), Ok(extension::EightBitMime));
        assert_eq!(SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::Size(42), extension::EightBitMime))
        }.supports_feature(extension::Size(0)), Ok(extension::Size(42)));
        assert!(SmtpServerInfo{
            name: String::from_str("name"),
            esmtp_features: Some(vec!(extension::EightBitMime))
        }.supports_feature(extension::Size(42)).is_err());
    }
}
