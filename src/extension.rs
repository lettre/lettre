// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! ESMTP features

use std::result::Result;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::collections::HashSet;

use response::Response;
use error::Error;
use authentication::Mecanism;

/// Supported ESMTP keywords
#[derive(PartialEq,Eq,Hash,Clone,Debug)]
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
    /// AUTH mecanism
    Authentication(Mecanism),
}

impl Display for Extension {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}",
            match *self {
                Extension::EightBitMime => "8BITMIME",
                Extension::SmtpUtfEight => "SMTPUTF8",
                Extension::StartTls => "STARTTLS",
                Extension::Authentication(_) => "AUTH",
            }
        )
    }
}

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
    pub esmtp_features: HashSet<Extension>,
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
    /// Parses a response to create a `ServerInfo`
    pub fn from_response(response: &Response) -> Result<ServerInfo, Error> {
        let name = match response.first_word() {
            Some(name) => name,
            None => return Err(Error::ResponseParsingError("Could not read server name"))
        };

        let mut esmtp_features: HashSet<Extension> = HashSet::new();

        for line in response.message() {

            let splitted : Vec<&str> = line.split_whitespace().collect();
            let _ = match (splitted[0], splitted.len()) {
                ("8BITMIME", 1) => {esmtp_features.insert(Extension::EightBitMime);},
                ("SMTPUTF8", 1) => {esmtp_features.insert(Extension::SmtpUtfEight);},
                ("STARTTLS", 1) => {esmtp_features.insert(Extension::StartTls);},
                ("AUTH", _) => {
                    for &mecanism in &splitted[1..] {
                        match mecanism {
                            "PLAIN" => {esmtp_features.insert(Extension::Authentication(Mecanism::Plain));},
                            "CRAM-MD5" => {esmtp_features.insert(Extension::Authentication(Mecanism::CramMd5));},
                            _ => (),
                        }
                    }
                },
                (_, _) => (),
            };
        }

        Ok(ServerInfo{
            name: name,
            esmtp_features: esmtp_features,
        })
    }

    /// Checks if the server supports an ESMTP feature
    pub fn supports_feature(&self, keyword: &Extension) -> bool {
        self.esmtp_features.contains(keyword)
    }

    /// Checks if the server supports an ESMTP feature
    pub fn supports_auth_mecanism(&self, mecanism: Mecanism) -> bool {
        self.esmtp_features.contains(&Extension::Authentication(mecanism))
    }
}

#[cfg(test)]
mod test {
	use std::collections::HashSet;
	
    use super::{ServerInfo, Extension};
    
    #[test]
    fn test_serverinfo_fmt() {
    	let mut eightbitmime = HashSet::new();
    	assert!(eightbitmime.insert(Extension::EightBitMime));
    	
    	let empty = HashSet::new();
    	
        assert_eq!(format!("{}", ServerInfo{
            name: "name".to_string(),
            esmtp_features: eightbitmime.clone()
        }), "name with {EightBitMime}".to_string());
        assert_eq!(format!("{}", ServerInfo{
            name: "name".to_string(),
            esmtp_features: empty,
        }), "name with no supported features".to_string());
    }
}
