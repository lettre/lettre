//! Provides authentication mecanisms

use std::fmt::{Display, Formatter};
use std::fmt;

use rustc_serialize::base64::{self, ToBase64, FromBase64};
use rustc_serialize::hex::ToHex;
use crypto::hmac::Hmac;
use crypto::md5::Md5;
use crypto::mac::Mac;

use transport::smtp::NUL;
use transport::error::Error;

/// Represents authentication mecanisms
#[derive(PartialEq,Eq,Copy,Clone,Hash,Debug)]
pub enum Mecanism {
    /// PLAIN authentication mecanism
    /// RFC 4616: https://tools.ietf.org/html/rfc4616
    Plain,
    /// CRAM-MD5 authentication mecanism
    /// RFC 2195: https://tools.ietf.org/html/rfc2195
    CramMd5,
}

impl Display for Mecanism {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,
               "{}",
               match *self {
                   Mecanism::Plain => "PLAIN",
                   Mecanism::CramMd5 => "CRAM-MD5",
               })
    }
}

impl Mecanism {
    /// Does the mecanism supports initial response
    pub fn supports_initial_response(&self) -> bool {
        match *self {
            Mecanism::Plain => true,
            Mecanism::CramMd5 => false,
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
            Mecanism::Plain => {
                match challenge {
                    Some(_) => Err(Error::ClientError("This mecanism does not expect a challenge")),
                    None => Ok(format!("{}{}{}{}", NUL, username, NUL, password)
                                   .as_bytes()
                                   .to_base64(base64::STANDARD)),
                }
            }
            Mecanism::CramMd5 => {
                let encoded_challenge = match challenge {
                    Some(challenge) => challenge,
                    None => return Err(Error::ClientError("This mecanism does expect a challenge")),
                };

                let decoded_challenge = match encoded_challenge.from_base64() {
                    Ok(challenge) => challenge,
                    Err(error) => return Err(Error::ChallengeParsingError(error)),
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
    use super::Mecanism;

    #[test]
    fn test_plain() {
        let mecanism = Mecanism::Plain;

        assert_eq!(mecanism.response("username", "password", None).unwrap(),
                   "AHVzZXJuYW1lAHBhc3N3b3Jk");
        assert!(mecanism.response("username", "password", Some("test")).is_err());
    }

    #[test]
    fn test_cram_md5() {
        let mecanism = Mecanism::CramMd5;

        assert_eq!(mecanism.response("alice",
                                     "wonderland",
                                     Some("PDE3ODkzLjEzMjA2NzkxMjNAdGVzc2VyYWN0LnN1c2FtLmluPg=="))
                           .unwrap(),
                   "YWxpY2UgNjRiMmE0M2MxZjZlZDY4MDZhOTgwOTE0ZTIzZTc1ZjA=");
        assert!(mecanism.response("alice", "wonderland", Some("tést")).is_err());
        assert!(mecanism.response("alice", "wonderland", None).is_err());
    }
}
