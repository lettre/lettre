use crate::message::{header::HeaderName, Headers, Message};
use base64::{decode, encode};
use ed25519_dalek::Signer;
use lazy_static::lazy_static;
use regex::Regex;
use rsa::{pkcs1::FromRsaPrivateKey, Hash, PaddingScheme, RsaPrivateKey};
use sha2::{Digest, Sha256};
use std::fmt::Display;
use std::fmt::Write;
use std::time::SystemTime;

/// Describe Dkim Canonicalization to apply to either body or headers
#[derive(Copy, Clone, Debug)]
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
#[derive(Copy, Clone, Debug)]
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

/// Describe the algorithm used for signing the message
#[derive(Copy, Clone)]
pub enum DkimSigningAlgorithm {
    Rsa,
    Ed25519,
}

impl Display for DkimSigningAlgorithm {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            DkimSigningAlgorithm::Rsa => write!(fmt, "rsa"),
            DkimSigningAlgorithm::Ed25519 => write!(fmt, "ed25519"),
        }
    }
}
/// A struct to describe Dkim configuration applied when signing a message
/// selector: the name of the key publied in DNS
/// domain: the domain for which we sign the message
/// private_key: private key in PKCS1 string format
/// headers: a list of headers name to be included in the signature. Signing of more than one
/// header with same name is not supported
/// canonicalization: the canonicalization to be applied on the message
/// pub signing_algorithm: the signing algorithm to be used when signing
#[derive(Clone)]
pub struct DkimConfig {
    selector: String,
    domain: String,
    private_key: String,
    headers: Vec<String>,
    canonicalization: DkimCanonicalization,
    pub signing_algorithm: DkimSigningAlgorithm,
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
            signing_algorithm: DkimSigningAlgorithm::Rsa,
        }
    }
}

/// Create a Headers struct with a Dkim-Signature Header created from given parameters
fn dkim_header_format(
    config: &DkimConfig,
    timestamp: String,
    headers_list: String,
    body_hash: String,
    signature: String,
) -> Headers {
    let mut headers = Headers::new();
    let header_name = match config.canonicalization.header {
        DkimCanonicalizationType::Simple => HeaderName::new_from_ascii_str("DKIM-Signature"),
        DkimCanonicalizationType::Relaxed => HeaderName::new_from_ascii_str("dkim-signature"),
    };
    headers.append_raw(header_name, format!("v=1; a={signing_algorithm}-sha256; d={domain}; s={selector}; c={canon}; q=dns/txt; t={timestamp}; h={headers_list}; bh={body_hash}; b={signature}",domain=config.domain, selector=config.selector,canon=config.canonicalization,timestamp=timestamp,headers_list=headers_list,body_hash=body_hash,signature=signature,signing_algorithm=config.signing_algorithm));
    headers
}

/// Canonicalize the body of an email
fn dkim_canonicalize_body(body: &[u8], canonicalization: DkimCanonicalizationType) -> String {
    let body = std::str::from_utf8(body).unwrap();
    lazy_static! {
        static ref RE: Regex = Regex::new("(\r\n)+$").unwrap();
    }
    match canonicalization {
        DkimCanonicalizationType::Simple => RE.replace(body, "\r\n").to_string(),
        DkimCanonicalizationType::Relaxed => {
            lazy_static! {
                static ref RE_DOUBLE_SPACE: Regex = Regex::new("[\\t ]+").unwrap();
                static ref RE_SPACE_EOL: Regex = Regex::new("[\t ]\r\n").unwrap();
            }
            let body = RE_DOUBLE_SPACE.replace_all(body, " ").to_string();
            let body = RE_SPACE_EOL.replace_all(&body, "\r\n").to_string();
            RE.replace(&body, "\r\n").to_string()
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
            lazy_static! {
                static ref RE_EOL: Regex = Regex::new("\r\n").unwrap();
                static ref RE_SPACES: Regex = Regex::new("[\\t ]+").unwrap();
            }
            let value = RE_EOL.replace_all(value, "").to_string();
            format!(
                "{}\r\n",
                RE_SPACES.replace_all(&value, " ").to_string().trim_end()
            )
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
            DkimCanonicalizationType::Relaxed => h.to_lowercase(),
        };
        if let Some(value) = mail_headers.get_raw(&h) {
            match canonicalization {
                DkimCanonicalizationType::Simple => signed_headers.append_raw(
                    HeaderName::new_from_ascii(h).unwrap(),
                    dkim_canonicalize_header_value(value, canonicalization),
                ),
                DkimCanonicalizationType::Relaxed => write!(
                    &mut signed_headers_relaxed,
                    "{}:{}",
                    h,
                    dkim_canonicalize_header_value(value, canonicalization)
                )
                .unwrap(),
            }
        }
    }
    match canonicalization {
        DkimCanonicalizationType::Simple => format!("{}", signed_headers),
        DkimCanonicalizationType::Relaxed => signed_headers_relaxed,
    }
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
    let body_hash = Sha256::digest(
        dkim_canonicalize_body(&message.body_raw(), dkim_config.canonicalization.body).as_bytes(),
    );
    let bh = encode(body_hash);
    let signed_headers_list = match dkim_config.canonicalization.header {
        DkimCanonicalizationType::Simple => dkim_config.headers.join(":"),
        DkimCanonicalizationType::Relaxed => dkim_config.headers.join(":").to_lowercase(),
    };
    let dkim_header = dkim_header_format(
        &dkim_config,
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
    let canonicalized_dkim_header = dkim_canonicalize_headers(
        vec!["DKIM-Signature".to_string()],
        &dkim_header,
        dkim_config.canonicalization.header,
    );
    let to_be_signed = signed_headers + &canonicalized_dkim_header;
    let to_be_signed = to_be_signed.trim_end();
    let hashed_headers = Sha256::digest(to_be_signed.as_bytes());
    let signature = match dkim_config.signing_algorithm {
        DkimSigningAlgorithm::Rsa => {
            let private_key = RsaPrivateKey::from_pkcs1_pem(&dkim_config.private_key).unwrap();
            encode(
                private_key
                    .sign(
                        PaddingScheme::new_pkcs1v15_sign(Some(Hash::SHA2_256)),
                        &hashed_headers,
                    )
                    .unwrap(),
            )
        }
        DkimSigningAlgorithm::Ed25519 => {
            let keypair =
                ed25519_dalek::Keypair::from_bytes(&decode(&dkim_config.private_key).unwrap())
                    .unwrap();
            encode(keypair.sign(&hashed_headers).to_bytes())
        }
    };
    let dkim_header =
        dkim_header_format(&dkim_config, timestamp, signed_headers_list, bh, signature);
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
        DkimCanonicalizationType,
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
