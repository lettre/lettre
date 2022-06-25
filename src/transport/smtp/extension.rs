//! ESMTP features

use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
    net::{Ipv4Addr, Ipv6Addr},
    result::Result,
};

use crate::transport::smtp::{
    authentication::Mechanism,
    error::{self, Error},
    response::Response,
    util::XText,
};

/// Client identifier, the parameter to `EHLO`
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ClientId {
    /// A fully-qualified domain name
    Domain(String),
    /// An IPv4 address
    Ipv4(Ipv4Addr),
    /// An IPv6 address
    Ipv6(Ipv6Addr),
}

const LOCALHOST_CLIENT: ClientId = ClientId::Ipv4(Ipv4Addr::new(127, 0, 0, 1));

impl Default for ClientId {
    fn default() -> Self {
        // https://tools.ietf.org/html/rfc5321#section-4.1.4
        //
        // The SMTP client MUST, if possible, ensure that the domain parameter
        // to the EHLO command is a primary host name as specified for this
        // command in Section 2.3.5.  If this is not possible (e.g., when the
        // client's address is dynamically assigned and the client does not have
        // an obvious name), an address literal SHOULD be substituted for the
        // domain name.
        #[cfg(feature = "hostname")]
        {
            hostname::get()
                .ok()
                .and_then(|s| s.into_string().map(Self::Domain).ok())
                .unwrap_or(LOCALHOST_CLIENT)
        }
        #[cfg(not(feature = "hostname"))]
        LOCALHOST_CLIENT
    }
}

impl Display for ClientId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Domain(ref value) => f.write_str(value),
            Self::Ipv4(ref value) => write!(f, "[{}]", value),
            Self::Ipv6(ref value) => write!(f, "[IPv6:{}]", value),
        }
    }
}

impl ClientId {
    #[doc(hidden)]
    #[deprecated(since = "0.10.0", note = "Please use ClientId::Domain(domain) instead")]
    /// Creates a new `ClientId` from a fully qualified domain name
    pub fn new(domain: String) -> Self {
        Self::Domain(domain)
    }
}

/// Supported ESMTP keywords
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Extension {
    /// 8BITMIME keyword
    ///
    /// Defined in [RFC 6152](https://tools.ietf.org/html/rfc6152)
    EightBitMime,
    /// SMTPUTF8 keyword
    ///
    /// Defined in [RFC 6531](https://tools.ietf.org/html/rfc6531)
    SmtpUtfEight,
    /// STARTTLS keyword
    ///
    /// Defined in [RFC 2487](https://tools.ietf.org/html/rfc2487)
    StartTls,
    /// AUTH mechanism
    Authentication(Mechanism),
}

impl Display for Extension {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Extension::EightBitMime => f.write_str("8BITMIME"),
            Extension::SmtpUtfEight => f.write_str("SMTPUTF8"),
            Extension::StartTls => f.write_str("STARTTLS"),
            Extension::Authentication(ref mechanism) => write!(f, "AUTH {}", mechanism),
        }
    }
}

/// Contains information about an SMTP server
#[derive(Clone, Debug, Eq, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ServerInfo {
    /// Server name
    ///
    /// The name given in the server banner
    name: String,
    /// ESMTP features supported by the server
    ///
    /// It contains the features supported by the server and known by the `Extension` module.
    features: HashSet<Extension>,
}

impl Display for ServerInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let features = if self.features.is_empty() {
            "no supported features".to_string()
        } else {
            format!("{:?}", self.features)
        };
        write!(f, "{} with {}", self.name, features)
    }
}

impl ServerInfo {
    /// Parses a EHLO response to create a `ServerInfo`
    pub fn from_response(response: &Response) -> Result<ServerInfo, Error> {
        let name = match response.first_word() {
            Some(name) => name,
            None => return Err(error::response("Could not read server name")),
        };

        let mut features: HashSet<Extension> = HashSet::new();

        for line in response.message() {
            if line.is_empty() {
                continue;
            }

            let mut split = line.split_whitespace();
            match split.next().unwrap() {
                "8BITMIME" => {
                    features.insert(Extension::EightBitMime);
                }
                "SMTPUTF8" => {
                    features.insert(Extension::SmtpUtfEight);
                }
                "STARTTLS" => {
                    features.insert(Extension::StartTls);
                }
                "AUTH" => {
                    for mechanism in split {
                        match mechanism {
                            "PLAIN" => {
                                features.insert(Extension::Authentication(Mechanism::Plain));
                            }
                            "LOGIN" => {
                                features.insert(Extension::Authentication(Mechanism::Login));
                            }
                            "XOAUTH2" => {
                                features.insert(Extension::Authentication(Mechanism::Xoauth2));
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            };
        }

        Ok(ServerInfo {
            name: name.to_string(),
            features,
        })
    }

    /// Checks if the server supports an ESMTP feature
    pub fn supports_feature(&self, keyword: Extension) -> bool {
        self.features.contains(&keyword)
    }

    /// Checks if the server supports an ESMTP feature
    pub fn supports_auth_mechanism(&self, mechanism: Mechanism) -> bool {
        self.features
            .contains(&Extension::Authentication(mechanism))
    }

    /// Gets a compatible mechanism from list
    pub fn get_auth_mechanism(&self, mechanisms: &[Mechanism]) -> Option<Mechanism> {
        for mechanism in mechanisms {
            if self.supports_auth_mechanism(*mechanism) {
                return Some(*mechanism);
            }
        }
        None
    }

    /// The name given in the server banner
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

/// A `MAIL FROM` extension parameter
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MailParameter {
    /// `BODY` parameter
    Body(MailBodyParameter),
    /// `SIZE` parameter
    Size(usize),
    /// `SMTPUTF8` parameter
    SmtpUtfEight,
    /// Custom parameter
    Other {
        /// Parameter keyword
        keyword: String,
        /// Parameter value
        value: Option<String>,
    },
}

impl Display for MailParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            MailParameter::Body(ref value) => write!(f, "BODY={}", value),
            MailParameter::Size(size) => write!(f, "SIZE={}", size),
            MailParameter::SmtpUtfEight => f.write_str("SMTPUTF8"),
            MailParameter::Other {
                ref keyword,
                value: Some(ref value),
            } => write!(f, "{}={}", keyword, XText(value)),
            MailParameter::Other {
                ref keyword,
                value: None,
            } => f.write_str(keyword),
        }
    }
}

/// Values for the `BODY` parameter to `MAIL FROM`
#[derive(PartialEq, Eq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MailBodyParameter {
    /// `7BIT`
    SevenBit,
    /// `8BITMIME`
    EightBitMime,
}

impl Display for MailBodyParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            MailBodyParameter::SevenBit => f.write_str("7BIT"),
            MailBodyParameter::EightBitMime => f.write_str("8BITMIME"),
        }
    }
}

/// A `RCPT TO` extension parameter
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RcptParameter {
    /// Custom parameter
    Other {
        /// Parameter keyword
        keyword: String,
        /// Parameter value
        value: Option<String>,
    },
}

impl Display for RcptParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            RcptParameter::Other {
                ref keyword,
                value: Some(ref value),
            } => write!(f, "{}={}", keyword, XText(value)),
            RcptParameter::Other {
                ref keyword,
                value: None,
            } => f.write_str(keyword),
        }
    }
}

#[cfg(test)]
mod test {

    use std::collections::HashSet;

    use super::*;
    use crate::transport::smtp::{
        authentication::Mechanism,
        response::{Category, Code, Detail, Response, Severity},
    };

    #[test]
    fn test_clientid_fmt() {
        assert_eq!(
            format!("{}", ClientId::Domain("test".to_string())),
            "test".to_string()
        );
        assert_eq!(format!("{}", LOCALHOST_CLIENT), "[127.0.0.1]".to_string());
    }

    #[test]
    fn test_extension_fmt() {
        assert_eq!(
            format!("{}", Extension::EightBitMime),
            "8BITMIME".to_string()
        );
        assert_eq!(
            format!("{}", Extension::Authentication(Mechanism::Plain)),
            "AUTH PLAIN".to_string()
        );
    }

    #[test]
    fn test_serverinfo_fmt() {
        let mut eightbitmime = HashSet::new();
        assert!(eightbitmime.insert(Extension::EightBitMime));

        assert_eq!(
            format!(
                "{}",
                ServerInfo {
                    name: "name".to_string(),
                    features: eightbitmime,
                }
            ),
            "name with {EightBitMime}".to_string()
        );

        let empty = HashSet::new();

        assert_eq!(
            format!(
                "{}",
                ServerInfo {
                    name: "name".to_string(),
                    features: empty,
                }
            ),
            "name with no supported features".to_string()
        );

        let mut plain = HashSet::new();
        assert!(plain.insert(Extension::Authentication(Mechanism::Plain)));

        assert_eq!(
            format!(
                "{}",
                ServerInfo {
                    name: "name".to_string(),
                    features: plain,
                }
            ),
            "name with {Authentication(Plain)}".to_string()
        );
    }

    #[test]
    fn test_serverinfo() {
        let response = Response::new(
            Code::new(
                Severity::PositiveCompletion,
                Category::Unspecified4,
                Detail::One,
            ),
            vec![
                "me".to_string(),
                "8BITMIME".to_string(),
                "SIZE 42".to_string(),
            ],
        );

        let mut features = HashSet::new();
        assert!(features.insert(Extension::EightBitMime));

        let server_info = ServerInfo {
            name: "me".to_string(),
            features,
        };

        assert_eq!(ServerInfo::from_response(&response).unwrap(), server_info);

        assert!(server_info.supports_feature(Extension::EightBitMime));
        assert!(!server_info.supports_feature(Extension::StartTls));

        let response2 = Response::new(
            Code::new(
                Severity::PositiveCompletion,
                Category::Unspecified4,
                Detail::One,
            ),
            vec![
                "me".to_string(),
                "AUTH PLAIN CRAM-MD5 XOAUTH2 OTHER".to_string(),
                "8BITMIME".to_string(),
                "SIZE 42".to_string(),
            ],
        );

        let mut features2 = HashSet::new();
        assert!(features2.insert(Extension::EightBitMime));
        assert!(features2.insert(Extension::Authentication(Mechanism::Plain),));
        assert!(features2.insert(Extension::Authentication(Mechanism::Xoauth2),));

        let server_info2 = ServerInfo {
            name: "me".to_string(),
            features: features2,
        };

        assert_eq!(ServerInfo::from_response(&response2).unwrap(), server_info2);

        assert!(server_info2.supports_feature(Extension::EightBitMime));
        assert!(server_info2.supports_auth_mechanism(Mechanism::Plain));
        assert!(!server_info2.supports_feature(Extension::StartTls));
    }
}
