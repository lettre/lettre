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
use std::fmt::{Display, Formatter};

use extension::Extension;

/// Contains information about an SMTP server
#[derive(Clone,Debug)]
pub struct ServerInfo {
    /// Server name
    ///
    /// The name given in the server banner
    pub name: String,
    /// ESMTP features supported by the server
    ///
    /// It contains the features supported by the server and known by the `Extension` module.
    pub esmtp_features: Vec<Extension>,
}

impl Display for ServerInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} with {}",
            self.name,
            match self.esmtp_features.is_empty() {
                true => "no supported features".to_string(),
                false => format! ("{:?}", self.esmtp_features),
            }
        )
    }
}

impl ServerInfo {
    /// Checks if the server supports an ESMTP feature
    pub fn supports_feature(&self, keyword: Extension) -> bool {
        self.esmtp_features.contains(&keyword)
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
            esmtp_features: vec![Extension::EightBitMime]
        }), "name with [EightBitMime]".to_string());
        assert_eq!(format!("{}", ServerInfo{
            name: "name".to_string(),
            esmtp_features: vec![Extension::EightBitMime]
        }), "name with [EightBitMime]".to_string());
        assert_eq!(format!("{}", ServerInfo{
            name: "name".to_string(),
            esmtp_features: vec![]
        }), "name with no supported features".to_string());
    }

    #[test]
    fn test_supports_feature() {
        assert!(ServerInfo{
            name: "name".to_string(),
            esmtp_features: vec![Extension::EightBitMime]
        }.supports_feature(Extension::EightBitMime));
        assert!(ServerInfo{
            name: "name".to_string(),
            esmtp_features: vec![Extension::PlainAuthentication, Extension::EightBitMime]
        }.supports_feature(Extension::EightBitMime));
        assert_eq!(ServerInfo{
            name: "name".to_string(),
            esmtp_features: vec![Extension::EightBitMime]
        }.supports_feature(Extension::PlainAuthentication), false);
    }
}
