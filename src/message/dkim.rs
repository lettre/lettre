use crate::message::{header::HeaderName, Headers, Message};
use base64::encode;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use regex::Regex;
use rsa::{pkcs1::FromRsaPrivateKey, Hash, PaddingScheme, RsaPrivateKey};
use std::fmt::Display;
use std::time::SystemTime;

/// Describe Dkim Canonicalization to apply to either body or headers
#[derive(Copy, Clone)]
pub enum DkimCanonicalizationType {
    Simple,
    Relaxed,
}

impl Display for DkimCanonicalizationType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            DkimCanonicalizationType::Simple => write!(fmt, "simple"),
            DkimCanonicalizationType::Relaxed => write!(fmt, "relaxed"),
        }
    }
}

/// Describe Canonicalization to be applied before signing
#[derive(Copy, Clone)]
pub struct DkimCanonicalization {
    header: DkimCanonicalizationType,
    body: DkimCanonicalizationType,
}

/// Format canonicalization to be shown in Dkim header
impl Display for DkimCanonicalization {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}/{}", self.header, self.body)
    }
}

/// A struct to describe Dkim configuration applied when signing a message
/// selector: the name of the key publied in DNS
/// domain: the domain for which we sign the message
/// private_key: private key in PKCS1 string format
/// headers: a list of headers name to be included in the signature. Signing of more than one
/// header with same name is not supported
/// canonicalization: the canonicalization to be applied on the message
#[derive(Clone)]
pub struct DkimConfig {
    selector: String,
    domain: String,
    private_key: String,
    headers: Vec<String>,
    canonicalization: DkimCanonicalization,
}

impl DkimConfig {
    /// Create a default signature configuration with a set of headers and "simple/relaxed"
    /// canonicalization
    pub fn default_config(selector: String, domain: String, private_key: String) -> DkimConfig {
        DkimConfig {
            selector,
            domain,
            private_key,
            headers: vec![
                "From".to_string(),
                "Subject".to_string(),
                "To".to_string(),
                "Date".to_string(),
            ],
            canonicalization: DkimCanonicalization {
                header: DkimCanonicalizationType::Simple,
                body: DkimCanonicalizationType::Relaxed,
            },
        }
    }
}

/// Create a Headers struct with a Dkim-Signature Header created from given parameters
fn dkim_header_format(
    domain: String,
    selector: String,
    canon: DkimCanonicalization,
    timestamp: String,
    headers_list: String,
    body_hash: String,
    signature: String,
) -> Headers {
    let mut headers = Headers::new();
    let header_name = match canon.header {
        DkimCanonicalizationType::Simple => HeaderName::new_from_ascii_str("DKIM-Signature"),
        DkimCanonicalizationType::Relaxed => HeaderName::new_from_ascii_str("dkim-signature"),
    };
    headers.append_raw(header_name, format!("v=1; a=rsa-sha256; d={domain}; s={selector}; c={canon}; q=dns/txt; t={timestamp}; h={headers_list}; bh={body_hash}; b={signature}",domain=domain, selector=selector,canon=canon,timestamp=timestamp,headers_list=headers_list,body_hash=body_hash,signature=signature));
    headers
}

/// Canonicalize the body of an email
fn dkim_canonicalize_body(body: &[u8], canonicalization: DkimCanonicalizationType) -> String {
    let body = std::str::from_utf8(body).unwrap();
    let re = Regex::new("(\r\n)+$").unwrap();
    match canonicalization {
        DkimCanonicalizationType::Simple => re.replace(body, "\r\n").to_string(),
        DkimCanonicalizationType::Relaxed => {
            let re_double_space = Regex::new("[\\t ]+").unwrap();
            let body = re_double_space.replace_all(body, " ").to_string();
            let re_space_eol = Regex::new("[\t ]\r\n").unwrap();
            let body = re_space_eol.replace_all(&body, "\r\n").to_string();
            re.replace(&body, "\r\n").to_string()
        }
    }
}

/// Canonicalize the value of an header
fn dkim_canonicalize_header_value(
    value: &str,
    canonicalization: DkimCanonicalizationType,
) -> String {
    match canonicalization {
        DkimCanonicalizationType::Simple => value.to_string(),
        DkimCanonicalizationType::Relaxed => {
            let re = Regex::new("\r\n").unwrap();
            let value = re.replace_all(value, "").to_string();
            let re = Regex::new("[\\t ]+").unwrap();
            format!("{}\r\n", re.replace_all(&value, " ").to_string().trim_end())
        }
    }
}

/// Canonicalize signed headers passed as headers_list among mail_headers using canonicalization
fn dkim_canonicalize_headers(
    headers_list: Vec<String>,
    mail_headers: &Headers,
    canonicalization: DkimCanonicalizationType,
) -> String {
    let mut signed_headers = Headers::new();
    let mut signed_headers_relaxed = String::new();
    for h in headers_list.into_iter() {
        let h = match canonicalization {
            DkimCanonicalizationType::Simple => h,
            DkimCanonicalizationType::Relaxed => {
                let mut ret = String::new();
                ret.push_str(&h);
                ret.to_lowercase()
            }
        };
        if let Some(value) = mail_headers.get_raw(&h) {
            match canonicalization {
                DkimCanonicalizationType::Simple => signed_headers.append_raw(
                    HeaderName::new_from_ascii(h).unwrap(),
                    dkim_canonicalize_header_value(value, canonicalization),
                ),
                DkimCanonicalizationType::Relaxed => signed_headers_relaxed.push_str(&format!(
                    "{}:{}",
                    h,
                    dkim_canonicalize_header_value(value, canonicalization)
                )),
            }
        }
    }
    match canonicalization {
        DkimCanonicalizationType::Simple => format!("{}", signed_headers),
        DkimCanonicalizationType::Relaxed => signed_headers_relaxed,
    }
}

/// Hash input using Sha256 into result
///
/// Example
/// ```
/// let mut hash=[b'1';32];
/// dkim_hash("test",&hash);
/// assert_eq(hash,[b'\0';32])
/// ```
fn dkim_hash(input: String, result: &mut [u8; 32]) {
    let mut hasher = Sha256::new();
    *result = [b'\0'; 32];
    hasher.input_str(&input);
    hasher.result(result);
}

/// Sign with Dkim a message by adding Dkim-Signture header created with configuration expressed by
/// dkim_config
///

pub fn dkim_sign(message: &mut Message, dkim_config: DkimConfig) {
    let timestamp = format!(
        "{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let headers = message.headers();
    let mut body_hash = [b'\0'; 32];
    dkim_hash(
        dkim_canonicalize_body(&message.body_raw(), dkim_config.canonicalization.body),
        &mut body_hash,
    );
    let bh = encode(body_hash);
    let signed_headers_list = match dkim_config.canonicalization.header {
        DkimCanonicalizationType::Simple => dkim_config.headers.join(":"),
        DkimCanonicalizationType::Relaxed => dkim_config.headers.join(":").to_lowercase(),
    };
    let dkim_header = dkim_header_format(
        dkim_config.domain.clone(),
        dkim_config.selector.clone(),
        dkim_config.canonicalization,
        timestamp.clone(),
        signed_headers_list.clone(),
        bh.clone(),
        "".to_string(),
    );
    let signed_headers = dkim_canonicalize_headers(
        dkim_config.headers.clone(),
        headers,
        dkim_config.canonicalization.header,
    );
    let private_key = RsaPrivateKey::from_pkcs1_pem(&dkim_config.private_key).unwrap();
    let canonicalized_dkim_header = dkim_canonicalize_headers(
        vec!["DKIM-Signature".to_string()],
        &dkim_header,
        dkim_config.canonicalization.header,
    );
    let canonicalized_dkim_header = canonicalized_dkim_header.trim_end();
    let to_be_signed = format!("{}{}", signed_headers, canonicalized_dkim_header);
    let to_be_signed = to_be_signed.trim_end();
    let mut hashed_headers = [b'\0'; 32];
    dkim_hash(to_be_signed.to_string(), &mut hashed_headers);
    let signature = encode(
        private_key
            .sign(
                PaddingScheme::new_pkcs1v15_sign(Some(Hash::SHA2_256)),
                &hashed_headers,
            )
            .unwrap(),
    );
    let dkim_header = dkim_header_format(
        dkim_config.domain,
        dkim_config.selector,
        dkim_config.canonicalization,
        timestamp,
        signed_headers_list,
        bh,
        signature,
    );
    let mut headers = headers.clone();
    headers.append_raw(
        HeaderName::new_from_ascii_str("DKIM-Signature"),
        dkim_header.get_raw("DKIM-Signature").unwrap().to_string(),
    );
    message.headers = headers;
}

#[cfg(test)]
mod test {
    use super::{
        super::header::HeaderName,
        super::{Header, Message},
        dkim_canonicalize_body, dkim_canonicalize_header_value, dkim_canonicalize_headers,
        dkim_hash, DkimCanonicalizationType,
    };
    use crate::StdError;

    #[derive(Clone)]
    struct TestHeader(String);

    impl Header for TestHeader {
        fn name() -> HeaderName {
            HeaderName::new_from_ascii_str("Test")
        }

        fn parse(s: &str) -> Result<Self, Box<dyn StdError + Send + Sync>> {
            Ok(Self(s.into()))
        }

        fn display(&self) -> String {
            self.0.clone()
        }
    }

    #[test]
    fn test_dkim_hash() {
        let mut hash = [b'1'; 32];
        let expected = [
            159, 134, 208, 129, 136, 76, 125, 101, 154, 47, 234, 160, 197, 90, 208, 21, 163, 191,
            79, 27, 43, 11, 130, 44, 209, 93, 108, 21, 176, 240, 10, 8,
        ];
        dkim_hash("test".to_string(), &mut hash);
        assert_eq!(hash, expected)
    }

    #[test]
    fn test_body_simple_canonicalize() {
        let body = "test\r\n\r\ntest   \ttest\r\n\r\n\r\n";
        let expected = "test\r\n\r\ntest   \ttest\r\n";
        assert_eq!(
            dkim_canonicalize_body(body.as_bytes(), DkimCanonicalizationType::Simple),
            expected.to_string()
        )
    }
    #[test]
    fn test_body_relaxed_canonicalize() {
        let body = "test\r\n\r\ntest   \ttest\r\n\r\n\r\n";
        let expected = "test\r\n\r\ntest test\r\n";
        assert_eq!(
            dkim_canonicalize_body(body.as_bytes(), DkimCanonicalizationType::Relaxed),
            expected.to_string()
        )
    }
    #[test]
    fn test_header_simple_canonicalize() {
        let value = "test\r\n\r\ntest   \ttest\r\n";
        let expected = "test\r\n\r\ntest   \ttest\r\n";
        assert_eq!(
            dkim_canonicalize_header_value(value, DkimCanonicalizationType::Simple),
            expected.to_string()
        )
    }
    #[test]
    fn test_header_relaxed_canonicalize() {
        let value = "test\r\n\r\ntest   \ttest\r\n";
        let expected = "testtest test\r\n";
        assert_eq!(
            dkim_canonicalize_header_value(value, DkimCanonicalizationType::Relaxed),
            expected.to_string()
        )
    }

    fn test_message() -> Message {
        Message::builder()
            .from("Test <test+ezrz@example.net>".parse().unwrap())
            .to("Test2 <test2@example.org>".parse().unwrap())
            .header(TestHeader("test  test very very long with spaces and extra spaces   \twill be folded to several lines ".to_string()))
            .subject("Test with utf-8 ë")
            .body("test\r\n\r\ntest   \ttest\r\n\r\n\r\n".to_string()).unwrap()
    }

    #[test]
    fn test_headers_simple_canonicalize() {
        let message = test_message();
        assert_eq!(dkim_canonicalize_headers(vec!["From".to_string(), "Test".to_string()], &message.headers, DkimCanonicalizationType::Simple),"From: Test <test+ezrz@example.net>\r\nTest: test  test very very long with spaces and extra spaces   \twill be \r\n folded to several lines \r\n")
    }
    #[test]
    fn test_headers_relaxed_canonicalize() {
        let message = test_message();
        assert_eq!(dkim_canonicalize_headers(vec!["From".to_string(), "Test".to_string()], &message.headers, DkimCanonicalizationType::Relaxed),"from:Test <test+ezrz@example.net>\r\ntest:test test very very long with spaces and extra spaces will be folded to several lines\r\n")
    }
}
