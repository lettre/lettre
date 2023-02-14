//! Provides limited SASL authentication mechanisms

use std::fmt::{self, Debug, Display, Formatter};

use crate::transport::smtp::error::{self, Error};

/// Accepted authentication mechanisms
///
/// Trying LOGIN last as it is deprecated.
pub const DEFAULT_MECHANISMS: &[Mechanism] = &[Mechanism::Plain, Mechanism::Login];

/// Contains user credentials
#[derive(PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Credentials {
    authentication_identity: String,
    secret: String,
}

impl Credentials {
    /// Create a `Credentials` struct from username and password
    pub fn new(username: String, password: String) -> Credentials {
        Credentials {
            authentication_identity: username,
            secret: password,
        }
    }
}

impl<S, T> From<(S, T)> for Credentials
where
    S: Into<String>,
    T: Into<String>,
{
    fn from((username, password): (S, T)) -> Self {
        Credentials::new(username.into(), password.into())
    }
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials").finish()
    }
}

/// Represents authentication mechanisms
#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mechanism {
    /// PLAIN authentication mechanism, defined in
    /// [RFC 4616](https://tools.ietf.org/html/rfc4616)
    Plain,
    /// LOGIN authentication mechanism
    /// Obsolete but needed for some providers (like office365)
    ///
    /// Defined in [draft-murchison-sasl-login-00](https://www.ietf.org/archive/id/draft-murchison-sasl-login-00.txt).
    Login,
    /// Non-standard XOAUTH2 mechanism, defined in
    /// [xoauth2-protocol](https://developers.google.com/gmail/imap/xoauth2-protocol)
    Xoauth2,
}

impl Display for Mechanism {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Mechanism::Plain => "PLAIN",
            Mechanism::Login => "LOGIN",
            Mechanism::Xoauth2 => "XOAUTH2",
        })
    }
}

impl Mechanism {
    /// Does the mechanism supports initial response
    pub fn supports_initial_response(self) -> bool {
        match self {
            Mechanism::Plain | Mechanism::Xoauth2 => true,
            Mechanism::Login => false,
        }
    }

    /// Returns the string to send to the server, using the provided username, password and
    /// challenge in some cases
    pub fn response(
        self,
        credentials: &Credentials,
        challenge: Option<&str>,
    ) -> Result<String, Error> {
        match self {
            Mechanism::Plain => match challenge {
                Some(_) => Err(error::client("This mechanism does not expect a challenge")),
                None => Ok(format!(
                    "\u{0}{}\u{0}{}",
                    credentials.authentication_identity, credentials.secret
                )),
            },
            Mechanism::Login => {
                let decoded_challenge = challenge
                    .ok_or_else(|| error::client("This mechanism does expect a challenge"))?;

                if ["User Name", "Username:", "Username"].contains(&decoded_challenge) {
                    return Ok(credentials.authentication_identity.clone());
                }

                if ["Password", "Password:"].contains(&decoded_challenge) {
                    return Ok(credentials.secret.clone());
                }

                Err(error::client("Unrecognized challenge"))
            }
            Mechanism::Xoauth2 => match challenge {
                Some(_) => Err(error::client("This mechanism does not expect a challenge")),
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

        let credentials = Credentials::new("username".to_owned(), "password".to_owned());

        assert_eq!(
            mechanism.response(&credentials, None).unwrap(),
            "\u{0}username\u{0}password"
        );
        assert!(mechanism.response(&credentials, Some("test")).is_err());
    }

    #[test]
    fn test_login() {
        let mechanism = Mechanism::Login;

        let credentials = Credentials::new("alice".to_owned(), "wonderland".to_owned());

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

        let credentials = Credentials::new(
            "username".to_owned(),
            "vF9dft4qmTc2Nvb3RlckBhdHRhdmlzdGEuY29tCg==".to_owned(),
        );

        assert_eq!(
            mechanism.response(&credentials, None).unwrap(),
            "user=username\x01auth=Bearer vF9dft4qmTc2Nvb3RlckBhdHRhdmlzdGEuY29tCg==\x01\x01"
        );
        assert!(mechanism.response(&credentials, Some("test")).is_err());
    }

    #[test]
    fn test_from_user_pass_for_credentials() {
        assert_eq!(
            Credentials::new("alice".to_owned(), "wonderland".to_owned()),
            Credentials::from(("alice", "wonderland"))
        );
    }
}
