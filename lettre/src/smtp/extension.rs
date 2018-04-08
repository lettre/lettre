//! ESMTP features

use hostname::get_hostname;
use smtp::authentication::Mechanism;
use smtp::error::Error;
use smtp::response::Response;
use smtp::util::XText;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::result::Result;

/// Default ehlo clinet id
pub const DEFAULT_EHLO_HOSTNAME: &str = "localhost";

/// Client identifier, the parameter to `EHLO`
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub enum ClientId {
    /// A fully-qualified domain name
    Domain(String),
    /// An IPv4 address
    Ipv4(Ipv4Addr),
    /// An IPv6 address
    Ipv6(Ipv6Addr),
}

impl Display for ClientId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ClientId::Domain(ref value) => f.write_str(value),
            ClientId::Ipv4(ref value) => write!(f, "{}", value),
            ClientId::Ipv6(ref value) => write!(f, "{}", value),
        }
    }
}

impl ClientId {
    /// Creates a new `ClientId` from a fully qualified domain name
    pub fn new(domain: String) -> ClientId {
        ClientId::Domain(domain)
    }

    /// Defines a `ClientId` with the current hostname, of `localhost` if hostname could not be
    /// found
    pub fn hostname() -> ClientId {
        ClientId::Domain(match get_hostname() {
            Some(name) => name,
            None => DEFAULT_EHLO_HOSTNAME.to_string(),
        })
    }
}

/// Supported ESMTP keywords
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
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
    /// AUTH mechanism
    Authentication(Mechanism),
}

impl Display for Extension {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Extension::EightBitMime => write!(f, "8BITMIME"),
            Extension::SmtpUtfEight => write!(f, "SMTPUTF8"),
            Extension::StartTls => write!(f, "STARTTLS"),
            Extension::Authentication(ref mechanism) => write!(f, "AUTH {}", mechanism),
        }
    }
}

/// Contains information about an SMTP server
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub struct ServerInfo {
    /// Server name
    ///
    /// The name given in the server banner
    pub name: String,
    /// ESMTP features supported by the server
    ///
    /// It contains the features supported by the server and known by the `Extension` module.
    pub features: HashSet<Extension>,
}

impl Display for ServerInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} with {}",
            self.name,
            if self.features.is_empty() {
                "no supported features".to_string()
            } else {
                format!("{:?}", self.features)
            }
        )
    }
}

impl ServerInfo {
    /// Parses a EHLO response to create a `ServerInfo`
    pub fn from_response(response: &Response) -> Result<ServerInfo, Error> {
        let name = match response.first_word() {
            Some(name) => name,
            None => return Err(Error::ResponseParsing("Could not read server name")),
        };

        let mut features: HashSet<Extension> = HashSet::new();

        for line in response.message.as_slice() {
            if line.is_empty() {
                continue;
            }

            let splitted: Vec<&str> = line.split_whitespace().collect();
            match splitted[0] {
                "8BITMIME" => {
                    features.insert(Extension::EightBitMime);
                }
                "SMTPUTF8" => {
                    features.insert(Extension::SmtpUtfEight);
                }
                "STARTTLS" => {
                    features.insert(Extension::StartTls);
                }
                "AUTH" => for &mechanism in &splitted[1..] {
                    match mechanism {
                        "PLAIN" => {
                            features.insert(Extension::Authentication(Mechanism::Plain));
                        }
                        "LOGIN" => {
                            features.insert(Extension::Authentication(Mechanism::Login));
                        }
                        #[cfg(feature = "crammd5-auth")]
                        "CRAM-MD5" => {
                            features.insert(Extension::Authentication(Mechanism::CramMd5));
                        }
                        _ => (),
                    }
                },
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
}

/// A `MAIL FROM` extension parameter
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub enum MailBodyParameter {
    /// `7BIT`
    SevenBit,
    /// `8BITMIME`
    EightBitMime,
}

impl Display for MailBodyParameter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MailBodyParameter::SevenBit => f.write_str("7BIT"),
            MailBodyParameter::EightBitMime => f.write_str("8BITMIME"),
        }
    }
}

/// A `RCPT TO` extension parameter
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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

    use super::{ClientId, Extension, ServerInfo};
    use smtp::authentication::Mechanism;
    use smtp::response::{Category, Code, Detail, Response, Severity};
    use std::collections::HashSet;

    #[test]
    fn test_clientid_fmt() {
        assert_eq!(
            format!("{}", ClientId::new("test".to_string())),
            "test".to_string()
        );
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
                    features: eightbitmime.clone(),
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
                    features: plain.clone(),
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
            features: features,
        };

        assert_eq!(ServerInfo::from_response(&response).unwrap(), server_info);

        assert!(server_info.supports_feature(Extension::EightBitMime));
        assert!(!server_info.supports_feature(Extension::StartTls));
        #[cfg(feature = "crammd5-auth")]
        assert!(!server_info.supports_auth_mechanism(Mechanism::CramMd5));

        let response2 = Response::new(
            Code::new(
                Severity::PositiveCompletion,
                Category::Unspecified4,
                Detail::One,
            ),
            vec![
                "me".to_string(),
                "AUTH PLAIN CRAM-MD5 OTHER".to_string(),
                "8BITMIME".to_string(),
                "SIZE 42".to_string(),
            ],
        );

        let mut features2 = HashSet::new();
        assert!(features2.insert(Extension::EightBitMime));
        assert!(features2.insert(Extension::Authentication(Mechanism::Plain),));
        #[cfg(feature = "crammd5-auth")]
        assert!(features2.insert(Extension::Authentication(Mechanism::CramMd5),));

        let server_info2 = ServerInfo {
            name: "me".to_string(),
            features: features2,
        };

        assert_eq!(ServerInfo::from_response(&response2).unwrap(), server_info2);

        assert!(server_info2.supports_feature(Extension::EightBitMime));
        assert!(server_info2.supports_auth_mechanism(Mechanism::Plain));
        #[cfg(feature = "crammd5-auth")]
        assert!(server_info2.supports_auth_mechanism(Mechanism::CramMd5));
        assert!(!server_info2.supports_feature(Extension::StartTls));
    }
}
