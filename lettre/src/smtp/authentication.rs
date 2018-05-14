//! Provides limited SASL authentication mechanisms

use smtp::error::Error;
use std::fmt::{self, Display, Formatter};

/// Accepted authentication mechanisms on an encrypted connection
/// Trying LOGIN last as it is deprecated.
pub const DEFAULT_ENCRYPTED_MECHANISMS: &[Mechanism] = &[Mechanism::Plain, Mechanism::Login];

/// Accepted authentication mechanisms on an unencrypted connection
pub const DEFAULT_UNENCRYPTED_MECHANISMS: &[Mechanism] = &[];

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
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub struct Credentials {
    authentication_identity: String,
    secret: String,
}

impl Credentials {
    /// Create a `Credentials` struct from username and password
    pub fn new(username: String, password: String) -> Credentials {
        Credentials { authentication_identity: username, secret: password }
    }
}

/// Represents authentication mechanisms
#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub enum Mechanism {
    /// PLAIN authentication mechanism
    /// RFC 4616: https://tools.ietf.org/html/rfc4616
    Plain,
    /// LOGIN authentication mechanism
    /// Obsolete but needed for some providers (like office365)
    /// https://www.ietf.org/archive/id/draft-murchison-sasl-login-00.txt
    Login,
    /// Non-standard XOAUTH2 mechanism
    /// https://developers.google.com/gmail/imap/xoauth2-protocol
    Xoauth2,
}

impl Display for Mechanism {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Mechanism::Plain => "PLAIN",
                Mechanism::Login => "LOGIN",
                Mechanism::Xoauth2 => "XOAUTH2",
            }
        )
    }
}

impl Mechanism {
    /// Does the mechanism supports initial response
    pub fn supports_initial_response(&self) -> bool {
        match *self {
            Mechanism::Plain | Mechanism::Xoauth2 => true,
            Mechanism::Login => false,
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
            Mechanism::Plain => match challenge {
                Some(_) => Err(Error::Client("This mechanism does not expect a challenge")),
                None => Ok(format!(
                    "\u{0}{}\u{0}{}",
                    credentials.authentication_identity, credentials.secret
                )),
            },
            Mechanism::Login => {
                let decoded_challenge = match challenge {
                    Some(challenge) => challenge,
                    None => return Err(Error::Client("This mechanism does expect a challenge")),
                };

                if vec!["User Name", "Username:", "Username"].contains(&decoded_challenge) {
                    return Ok(credentials.authentication_identity.to_string());
                }

                if vec!["Password", "Password:"].contains(&decoded_challenge) {
                    return Ok(credentials.secret.to_string());
                }

                Err(Error::Client("Unrecognized challenge"))
            },
            Mechanism::Xoauth2 => match challenge {
                Some(_) => Err(Error::Client("This mechanism does not expect a challenge")),
                None => Ok(format!(
                    "user={}\x01auth=Bearer {}\x01\x01",
                    credentials.authentication_identity, credentials.secret
                )),
            },
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
    fn test_xoauth2() {
        let mechanism = Mechanism::Xoauth2;

        let credentials = Credentials::new("username".to_string(), "vF9dft4qmTc2Nvb3RlckBhdHRhdmlzdGEuY29tCg==".to_string());

        assert_eq!(
            mechanism.response(&credentials, None).unwrap(),
            "user=username\x01auth=Bearer vF9dft4qmTc2Nvb3RlckBhdHRhdmlzdGEuY29tCg==\x01\x01"
        );
        assert!(mechanism.response(&credentials, Some("test")).is_err());
    }
}
