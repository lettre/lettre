//! Provides authentication mechanisms

use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::md5::Md5;
use hex::ToHex;
use smtp::NUL;
use smtp::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

/// Convertable to user credentials
pub trait IntoCredentials {
    /// Converts to a `Credentials` struct
    fn into_credentials(self) -> Credentials;
}

impl IntoCredentials for Credentials {
    fn into_credentials(self) -> Credentials {
        self
    }
}

impl<S: Into<String>, T: Into<String>> IntoCredentials for (S, T) {
    fn into_credentials(self) -> Credentials {
        let (username, password) = self;
        Credentials::new(username.into(), password.into())
    }
}

/// Contains user credentials
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    /// Create a `Credentials` struct from username and password
    pub fn new(username: String, password: String) -> Credentials {
        Credentials {
            username: username,
            password: password,
        }
    }
}

/// Represents authentication mechanisms
#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub enum Mechanism {
    /// PLAIN authentication mechanism
    /// RFC 4616: https://tools.ietf.org/html/rfc4616
    Plain,
    /// LOGIN authentication mechanism
    /// Obsolete but needed for some providers (like office365)
    /// https://www.ietf.org/archive/id/draft-murchison-sasl-login-00.txt
    Login,
    /// CRAM-MD5 authentication mechanism
    /// RFC 2195: https://tools.ietf.org/html/rfc2195
    CramMd5,
}

impl Display for Mechanism {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Mechanism::Plain => "PLAIN",
                Mechanism::Login => "LOGIN",
                Mechanism::CramMd5 => "CRAM-MD5",
            }
        )
    }
}

impl Mechanism {
    /// Does the mechanism supports initial response
    pub fn supports_initial_response(&self) -> bool {
        match *self {
            Mechanism::Plain => true,
            Mechanism::Login |
            Mechanism::CramMd5 => false,
        }
    }

    /// Returns the string to send to the server, using the provided username, password and
    /// challenge in some cases
    pub fn response(
        &self,
        credentials: &Credentials,
        challenge: Option<&str>,
    ) -> Result<String, Error> {
        match *self {
            Mechanism::Plain => {
                match challenge {
                    Some(_) => Err(Error::Client("This mechanism does not expect a challenge")),
                    None => Ok(format!(
                        "{}{}{}{}",
                        NUL,
                        credentials.username,
                        NUL,
                        credentials.password
                    )),
                }
            }
            Mechanism::Login => {
                let decoded_challenge = match challenge {
                    Some(challenge) => challenge,
                    None => return Err(Error::Client("This mechanism does expect a challenge")),
                };

                if vec!["User Name", "Username:", "Username"].contains(&decoded_challenge) {
                    return Ok(credentials.username.to_string());
                }

                if vec!["Password", "Password:"].contains(&decoded_challenge) {
                    return Ok(credentials.password.to_string());
                }

                Err(Error::Client("Unrecognized challenge"))
            }
            Mechanism::CramMd5 => {
                let decoded_challenge = match challenge {
                    Some(challenge) => challenge,
                    None => return Err(Error::Client("This mechanism does expect a challenge")),
                };

                let mut hmac = Hmac::new(Md5::new(), credentials.password.as_bytes());
                hmac.input(decoded_challenge.as_bytes());

                Ok(format!(
                    "{} {}",
                    credentials.username,
                    hmac.result().code().to_hex()
                ))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Credentials, Mechanism};

    #[test]
    fn test_plain() {
        let mechanism = Mechanism::Plain;

        let credentials = Credentials::new("username".to_string(), "password".to_string());

        assert_eq!(
            mechanism.response(&credentials, None).unwrap(),
            "\u{0}username\u{0}password"
        );
        assert!(mechanism.response(&credentials, Some("test")).is_err());
    }

    #[test]
    fn test_login() {
        let mechanism = Mechanism::Login;

        let credentials = Credentials::new("alice".to_string(), "wonderland".to_string());

        assert_eq!(
            mechanism.response(&credentials, Some("Username")).unwrap(),
            "alice"
        );
        assert_eq!(
            mechanism.response(&credentials, Some("Password")).unwrap(),
            "wonderland"
        );
        assert!(mechanism.response(&credentials, None).is_err());
    }

    #[test]
    fn test_cram_md5() {
        let mechanism = Mechanism::CramMd5;

        let credentials = Credentials::new("alice".to_string(), "wonderland".to_string());

        assert_eq!(
            mechanism
                .response(
                    &credentials,
                    Some("PDE3ODkzLjEzMjA2NzkxMjNAdGVzc2VyYWN0LnN1c2FtLmluPg=="),
                )
                .unwrap(),
            "alice a540ebe4ef2304070bbc3c456c1f64c0"
        );
        assert!(mechanism.response(&credentials, None).is_err());
    }
}
