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
        match *self {
            Extension::EightBitMime => write!(f, "{}", "8BITMIME"),
            Extension::SmtpUtfEight => write!(f, "{}", "SMTPUTF8"),
            Extension::StartTls => write!(f, "{}", "STARTTLS"),
            Extension::Authentication(ref mecanism) => write!(f, "{} {}", "AUTH", mecanism),
        }
    }
}

/// Contains information about an SMTP server
#[derive(Clone,Debug,Eq,PartialEq)]
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
        write!(f,
               "{} with {}",
               self.name,
               match self.features.is_empty() {
                   true => "no supported features".to_string(),
                   false => format!("{:?}", self.features),
               })
    }
}

impl ServerInfo {
    /// Parses a response to create a `ServerInfo`
    pub fn from_response(response: &Response) -> Result<ServerInfo, Error> {
        let name = match response.first_word() {
            Some(name) => name,
            None => return Err(Error::ResponseParsingError("Could not read server name")),
        };

        let mut features: HashSet<Extension> = HashSet::new();

        for line in response.message() {

            let splitted: Vec<&str> = line.split_whitespace().collect();
            let _ = match splitted[0] {
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
                    for &mecanism in &splitted[1..] {
                        match mecanism {
                            "PLAIN" => {
                                features.insert(Extension::Authentication(Mecanism::Plain));
                            }
                            "CRAM-MD5" => {
                                features.insert(Extension::Authentication(Mecanism::CramMd5));
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            };
        }

        Ok(ServerInfo {
            name: name,
            features: features,
        })
    }

    /// Checks if the server supports an ESMTP feature
    pub fn supports_feature(&self, keyword: &Extension) -> bool {
        self.features.contains(keyword)
    }

    /// Checks if the server supports an ESMTP feature
    pub fn supports_auth_mecanism(&self, mecanism: Mecanism) -> bool {
        self.features.contains(&Extension::Authentication(mecanism))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::{ServerInfo, Extension};
    use authentication::Mecanism;
    use response::{Code, Response, Severity, Category};

    #[test]
    fn test_extension_fmt() {
        assert_eq!(format!("{}", Extension::EightBitMime),
                   "8BITMIME".to_string());
        assert_eq!(format!("{}", Extension::Authentication(Mecanism::Plain)),
                   "AUTH PLAIN".to_string());
    }

    #[test]
    fn test_serverinfo_fmt() {
        let mut eightbitmime = HashSet::new();
        assert!(eightbitmime.insert(Extension::EightBitMime));

        assert_eq!(format!("{}",
                           ServerInfo {
                               name: "name".to_string(),
                               features: eightbitmime.clone(),
                           }),
                   "name with {EightBitMime}".to_string());

        let empty = HashSet::new();

        assert_eq!(format!("{}",
                           ServerInfo {
                               name: "name".to_string(),
                               features: empty,
                           }),
                   "name with no supported features".to_string());

        let mut plain = HashSet::new();
        assert!(plain.insert(Extension::Authentication(Mecanism::Plain)));

        assert_eq!(format!("{}",
                           ServerInfo {
                               name: "name".to_string(),
                               features: plain.clone(),
                           }),
                   "name with {Authentication(Plain)}".to_string());
    }

    #[test]
    fn test_serverinfo() {
        let response = Response::new(Code::new(Severity::PositiveCompletion,
                                               Category::Unspecified4,
                                               1),
                                     vec!["me".to_string(),
                                          "8BITMIME".to_string(),
                                          "SIZE 42".to_string()]);

        let mut features = HashSet::new();
        assert!(features.insert(Extension::EightBitMime));

        let server_info = ServerInfo {
            name: "me".to_string(),
            features: features,
        };

        assert_eq!(ServerInfo::from_response(&response).unwrap(), server_info);

        assert!(server_info.supports_feature(&Extension::EightBitMime));
        assert!(!server_info.supports_feature(&Extension::StartTls));
        assert!(!server_info.supports_auth_mecanism(Mecanism::CramMd5));

        let response2 = Response::new(Code::new(Severity::PositiveCompletion,
                                                Category::Unspecified4,
                                                1),
                                      vec!["me".to_string(),
                                           "AUTH PLAIN CRAM-MD5 OTHER".to_string(),
                                           "8BITMIME".to_string(),
                                           "SIZE 42".to_string()]);

        let mut features2 = HashSet::new();
        assert!(features2.insert(Extension::EightBitMime));
        assert!(features2.insert(Extension::Authentication(Mecanism::Plain)));
        assert!(features2.insert(Extension::Authentication(Mecanism::CramMd5)));

        let server_info2 = ServerInfo {
            name: "me".to_string(),
            features: features2,
        };

        assert_eq!(ServerInfo::from_response(&response2).unwrap(), server_info2);

        assert!(server_info2.supports_feature(&Extension::EightBitMime));
        assert!(server_info2.supports_auth_mecanism(Mecanism::Plain));
        assert!(server_info2.supports_auth_mecanism(Mecanism::CramMd5));
        assert!(!server_info2.supports_feature(&Extension::StartTls));
    }
}
