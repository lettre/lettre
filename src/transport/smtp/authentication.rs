//! Provides authentication mechanisms

use std::fmt;
use std::fmt::{Display, Formatter};

use rustc_serialize::base64::{self, FromBase64, ToBase64};
use rustc_serialize::hex::ToHex;
use crypto::hmac::Hmac;
use crypto::md5::Md5;
use crypto::mac::Mac;

use transport::smtp::NUL;
use transport::error::Error;

/// Represents authentication mechanisms
#[derive(PartialEq,Eq,Copy,Clone,Hash,Debug)]
pub enum Mechanism {
    /// PLAIN authentication mechanism
    /// RFC 4616: https://tools.ietf.org/html/rfc4616
    Plain,
    /// CRAM-MD5 authentication mechanism
    /// RFC 2195: https://tools.ietf.org/html/rfc2195
    CramMd5,
}

impl Display for Mechanism {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,
               "{}",
               match *self {
                   Mechanism::Plain => "PLAIN",
                   Mechanism::CramMd5 => "CRAM-MD5",
               })
    }
}

impl Mechanism {
    /// Does the mechanism supports initial response
    pub fn supports_initial_response(&self) -> bool {
        match *self {
            Mechanism::Plain => true,
            Mechanism::CramMd5 => false,
        }
    }

    /// Returns the string to send to the server, using the provided username, password and
    /// challenge in some cases
    pub fn response(&self,
                    username: &str,
                    password: &str,
                    challenge: Option<&str>)
                    -> Result<String, Error> {
        match *self {
            Mechanism::Plain => {
                match challenge {
                    Some(_) => Err(Error::Client("This mechanism does not expect a challenge")),
                    None => {
                        Ok(format!("{}{}{}{}", NUL, username, NUL, password)
                            .as_bytes()
                            .to_base64(base64::STANDARD))
                    }
                }
            }
            Mechanism::CramMd5 => {
                let encoded_challenge = match challenge {
                    Some(challenge) => challenge,
                    None => return Err(Error::Client("This mechanism does expect a challenge")),
                };

                let decoded_challenge = match encoded_challenge.from_base64() {
                    Ok(challenge) => challenge,
                    Err(error) => return Err(Error::ChallengeParsing(error)),
                };

                let mut hmac = Hmac::new(Md5::new(), password.as_bytes());
                hmac.input(&decoded_challenge);

                Ok(format!("{} {}", username, hmac.result().code().to_hex())
                    .as_bytes()
                    .to_base64(base64::STANDARD))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Mechanism;

    #[test]
    fn test_plain() {
        let mechanism = Mechanism::Plain;

        assert_eq!(mechanism.response("username", "password", None).unwrap(),
                   "AHVzZXJuYW1lAHBhc3N3b3Jk");
        assert!(mechanism.response("username", "password", Some("test")).is_err());
    }

    #[test]
    fn test_cram_md5() {
        let mechanism = Mechanism::CramMd5;

        assert_eq!(mechanism.response("alice",
                                 "wonderland",
                                 Some("PDE3ODkzLjEzMjA2NzkxMjNAdGVzc2VyYWN0LnN1c2FtLmluPg=="))
                       .unwrap(),
                   "YWxpY2UgNjRiMmE0M2MxZjZlZDY4MDZhOTgwOTE0ZTIzZTc1ZjA=");
        assert!(mechanism.response("alice", "wonderland", Some("tést")).is_err());
        assert!(mechanism.response("alice", "wonderland", None).is_err());
    }
}
