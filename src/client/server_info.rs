// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Information about a server

use std::fmt;
use std::fmt::{Show, Formatter};

use extension::Extension;

/// Contains information about an SMTP server
#[deriving(Clone)]
pub struct ServerInfo {
    /// Server name
    ///
    /// The name given in the server banner
    pub name: String,
    /// ESMTP features supported by the server
    ///
    /// It contains the features supported by the server and known by the `Extension` module.
    /// The `None` value means the server does not support ESMTP.
    pub esmtp_features: Option<Vec<Extension>>,
}

impl Show for ServerInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write(
            format!("{} with {}",
                self.name,
                match self.esmtp_features {
                    Some(ref features) => features.to_string(),
                    None => "no supported features".to_string(),
                }
            ).as_bytes()
        )
    }
}

impl ServerInfo {
    /// Checks if the server supports an ESMTP feature
    pub fn supports_feature(&self, keyword: Extension) -> Option<Extension> {
        match self.esmtp_features {
            Some(ref esmtp_features) => {
                for feature in esmtp_features.iter() {
                    if keyword.same_extension_as(feature) {
                        return Some(*feature);
                    }
                }
                None
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::ServerInfo;
    use extension::Extension;

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec![Extension::EightBitMime])
        }), "name with [8BITMIME]".to_string());
        assert_eq!(format!("{}", ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec![Extension::EightBitMime, Extension::Size(42)])
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
            esmtp_features: Some(vec![Extension::EightBitMime])
        }.supports_feature(Extension::EightBitMime), Some(Extension::EightBitMime));
        assert_eq!(ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec![Extension::Size(42), Extension::EightBitMime])
        }.supports_feature(Extension::EightBitMime), Some(Extension::EightBitMime));
        assert_eq!(ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec![Extension::Size(42), Extension::EightBitMime])
        }.supports_feature(Extension::Size(0)), Some(Extension::Size(42)));
        assert!(ServerInfo{
            name: "name".to_string(),
            esmtp_features: Some(vec![Extension::EightBitMime])
        }.supports_feature(Extension::Size(42)).is_none());
    }
}
