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

use extension::Extension;

/// Information about an SMTP server
#[deriving(Clone)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// ESMTP features supported by the server
    /// The `None` value means the server does not support ESMTP
    pub esmtp_features: Option<Vec<Extension>>
}

impl Show for ServerInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write(
            format!("{} with {}",
                self.name,
                match self.esmtp_features.clone() {
                    Some(features) => features.to_string(),
                    None => "no supported features".to_string()
                }
            ).as_bytes()
        )
    }
}

impl ServerInfo {
    /// Checks if the server supports an ESMTP feature
    pub fn supports_feature(&self, keyword: Extension) -> Option<Extension> {
        match self.esmtp_features.clone() {
            Some(esmtp_features) => {
                for feature in esmtp_features.iter() {
                    if keyword.same_extension_as(*feature) {
                        return Some(*feature);
                    }
                }
                None
            },
            None => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::ServerInfo;
    use extension;

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec!(extension::EightBitMime))
        }), "name with [8BITMIME]".to_string());
        assert_eq!(format!("{}", ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec!(extension::EightBitMime, extension::Size(42)))
        }), "name with [8BITMIME, SIZE=42]".to_string());
        assert_eq!(format!("{}", ServerInfo{
            name: "name".to_string(),
            esmtp_features: None
        }), "name with no supported features".to_string());
    }

    #[test]
    fn test_supports_feature() {
        assert_eq!(ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec!(extension::EightBitMime))
        }.supports_feature(extension::EightBitMime), Some(extension::EightBitMime));
        assert_eq!(ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec!(extension::Size(42), extension::EightBitMime))
        }.supports_feature(extension::EightBitMime), Some(extension::EightBitMime));
        assert_eq!(ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec!(extension::Size(42), extension::EightBitMime))
        }.supports_feature(extension::Size(0)), Some(extension::Size(42)));
        assert!(ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec!(extension::EightBitMime))
        }.supports_feature(extension::Size(42)).is_none());
    }
}
