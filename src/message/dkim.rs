use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display, Write},
    iter::IntoIterator,
    time::SystemTime,
};

use ed25519_dalek::Signer;
use once_cell::sync::Lazy;
use regex::{bytes::Regex as BRegex, Regex};
use rsa::{pkcs1::DecodeRsaPrivateKey, Hash, PaddingScheme, RsaPrivateKey};
use sha2::{Digest, Sha256};

use crate::message::{
    header::{HeaderName, HeaderValue},
    Headers, Message,
};

/// Describe Dkim Canonicalization to apply to either body or headers
#[derive(Copy, Clone, Debug)]
pub enum DkimCanonicalizationType {
    Simple,
    Relaxed,
}

impl Display for DkimCanonicalizationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DkimCanonicalizationType::Simple => "simple",
            DkimCanonicalizationType::Relaxed => "relaxed",
        })
    }
}

/// Describe Canonicalization to be applied before signing
#[derive(Copy, Clone, Debug)]
pub struct DkimCanonicalization {
    pub header: DkimCanonicalizationType,
    pub body: DkimCanonicalizationType,
}

impl Default for DkimCanonicalization {
    fn default() -> Self {
        DkimCanonicalization {
            header: DkimCanonicalizationType::Simple,
            body: DkimCanonicalizationType::Relaxed,
        }
    }
}

/// Format canonicalization to be shown in Dkim header
impl Display for DkimCanonicalization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.header, self.body)
    }
}

/// Describe the algorithm used for signing the message
#[derive(Copy, Clone, Debug)]
pub enum DkimSigningAlgorithm {
    Rsa,
    Ed25519,
}

impl Display for DkimSigningAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DkimSigningAlgorithm::Rsa => "rsa",
            DkimSigningAlgorithm::Ed25519 => "ed25519",
        })
    }
}

/// Describe DkimSigning key error
#[derive(Debug)]
pub struct DkimSigningKeyError(InnerDkimSigningKeyError);

#[derive(Debug)]
enum InnerDkimSigningKeyError {
    Base64(base64::DecodeError),
    Rsa(rsa::pkcs1::Error),
    Ed25519(ed25519_dalek::ed25519::Error),
}

impl Display for DkimSigningKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match &self.0 {
            InnerDkimSigningKeyError::Base64(_err) => "base64 decode error",
            InnerDkimSigningKeyError::Rsa(_err) => "rsa decode error",
            InnerDkimSigningKeyError::Ed25519(_err) => "ed25519 decode error",
        })
    }
}

impl StdError for DkimSigningKeyError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match &self.0 {
            InnerDkimSigningKeyError::Base64(err) => &*err,
            InnerDkimSigningKeyError::Rsa(err) => &*err,
            InnerDkimSigningKeyError::Ed25519(err) => &*err,
        })
    }
}

/// Describe a signing key to be carried by DkimConfig struct
#[derive(Debug)]
pub struct DkimSigningKey(InnerDkimSigningKey);

#[derive(Debug)]
enum InnerDkimSigningKey {
    Rsa(RsaPrivateKey),
    Ed25519(ed25519_dalek::Keypair),
}

impl DkimSigningKey {
    pub fn new(
        private_key: String,
        algorithm: DkimSigningAlgorithm,
    ) -> Result<DkimSigningKey, DkimSigningKeyError> {
        Ok(Self(match algorithm {
            DkimSigningAlgorithm::Rsa => InnerDkimSigningKey::Rsa(
                RsaPrivateKey::from_pkcs1_pem(&private_key)
                    .map_err(|err| DkimSigningKeyError(InnerDkimSigningKeyError::Rsa(err)))?,
            ),
            DkimSigningAlgorithm::Ed25519 => {
                InnerDkimSigningKey::Ed25519(
                    ed25519_dalek::Keypair::from_bytes(&base64::decode(private_key).map_err(
                        |err| DkimSigningKeyError(InnerDkimSigningKeyError::Base64(err)),
                    )?)
                    .map_err(|err| DkimSigningKeyError(InnerDkimSigningKeyError::Ed25519(err)))?,
                )
            }
        }))
    }
    fn get_signing_algorithm(&self) -> DkimSigningAlgorithm {
        match self.0 {
            InnerDkimSigningKey::Rsa(_) => DkimSigningAlgorithm::Rsa,
            InnerDkimSigningKey::Ed25519(_) => DkimSigningAlgorithm::Ed25519,
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
#[derive(Debug)]
pub struct DkimConfig {
    selector: String,
    domain: String,
    private_key: DkimSigningKey,
    headers: Vec<HeaderName>,
    canonicalization: DkimCanonicalization,
}

impl DkimConfig {
    /// Create a default signature configuration with a set of headers and "simple/relaxed"
    /// canonicalization
    pub fn default_config(
        selector: String,
        domain: String,
        private_key: DkimSigningKey,
    ) -> DkimConfig {
        DkimConfig {
            selector,
            domain,
            private_key,
            headers: vec![
                HeaderName::new_from_ascii_str("From"),
                HeaderName::new_from_ascii_str("Subject"),
                HeaderName::new_from_ascii_str("To"),
                HeaderName::new_from_ascii_str("Date"),
            ],
            canonicalization: DkimCanonicalization {
                header: DkimCanonicalizationType::Simple,
                body: DkimCanonicalizationType::Relaxed,
            },
        }
    }

    /// Create a DkimConfig
    pub fn new(
        selector: String,
        domain: String,
        private_key: DkimSigningKey,
        headers: Vec<HeaderName>,
        canonicalization: DkimCanonicalization,
    ) -> DkimConfig {
        DkimConfig {
            selector,
            domain,
            private_key,
            headers,
            canonicalization,
        }
    }
}

/// Create a Headers struct with a Dkim-Signature Header created from given parameters
fn dkim_header_format(
    config: &DkimConfig,
    timestamp: u64,
    headers_list: &str,
    body_hash: &str,
    signature: &str,
) -> Headers {
    let mut headers = Headers::new();
    let header_name =
        dkim_canonicalize_header_tag("DKIM-Signature", config.canonicalization.header);
    let header_name = HeaderName::new_from_ascii(header_name.into()).unwrap();
    headers.insert_raw(HeaderValue::new(header_name, format!("v=1; a={signing_algorithm}-sha256; d={domain}; s={selector}; c={canon}; q=dns/txt; t={timestamp}; h={headers_list}; bh={body_hash}; b={signature}",domain=config.domain, selector=config.selector,canon=config.canonicalization,timestamp=timestamp,headers_list=headers_list,body_hash=body_hash,signature=signature,signing_algorithm=config.private_key.get_signing_algorithm())));
    headers
}

/// Canonicalize the body of an email
fn dkim_canonicalize_body(
    body: &[u8],
    canonicalization: DkimCanonicalizationType,
) -> Cow<'_, [u8]> {
    static RE: Lazy<BRegex> = Lazy::new(|| BRegex::new("(\r\n)+$").unwrap());
    static RE_DOUBLE_SPACE: Lazy<BRegex> = Lazy::new(|| BRegex::new("[\\t ]+").unwrap());
    static RE_SPACE_EOL: Lazy<BRegex> = Lazy::new(|| BRegex::new("[\t ]\r\n").unwrap());
    match canonicalization {
        DkimCanonicalizationType::Simple => RE.replace(body, &b"\r\n"[..]),
        DkimCanonicalizationType::Relaxed => {
            let body = RE_DOUBLE_SPACE.replace_all(body, &b" "[..]);
            let body = match RE_SPACE_EOL.replace_all(&body, &b"\r\n"[..]) {
                Cow::Borrowed(_body) => body,
                Cow::Owned(body) => Cow::Owned(body),
            };
            match RE.replace(&body, &b"\r\n"[..]) {
                Cow::Borrowed(_body) => body,
                Cow::Owned(body) => Cow::Owned(body),
            }
        }
    }
}

/// Canonicalize the value of an header
fn dkim_canonicalize_header_value(
    value: &str,
    canonicalization: DkimCanonicalizationType,
) -> Cow<'_, str> {
    match canonicalization {
        DkimCanonicalizationType::Simple => Cow::Borrowed(value),
        DkimCanonicalizationType::Relaxed => {
            static RE_EOL: Lazy<Regex> = Lazy::new(|| Regex::new("\r\n").unwrap());
            static RE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new("[\\t ]+").unwrap());
            let value = RE_EOL.replace_all(value, "");
            Cow::Owned(format!(
                "{}\r\n",
                RE_SPACES.replace_all(&value, " ").trim_end()
            ))
        }
    }
}

/// Canonicalize header tag
fn dkim_canonicalize_header_tag(
    name: &str,
    canonicalization: DkimCanonicalizationType,
) -> Cow<'_, str> {
    match canonicalization {
        DkimCanonicalizationType::Simple => Cow::Borrowed(name),
        DkimCanonicalizationType::Relaxed => Cow::Owned(name.to_lowercase()),
    }
}

/// Canonicalize signed headers passed as headers_list among mail_headers using canonicalization
fn dkim_canonicalize_headers<'a>(
    headers_list: impl IntoIterator<Item = &'a str>,
    mail_headers: &Headers,
    canonicalization: DkimCanonicalizationType,
) -> String {
    match canonicalization {
        DkimCanonicalizationType::Simple => {
            let mut signed_headers = Headers::new();

            for h in headers_list {
                let h = dkim_canonicalize_header_tag(h, canonicalization);
                if let Some(value) = mail_headers.get_raw(&h) {
                    signed_headers.insert_raw(HeaderValue::new(
                        HeaderName::new_from_ascii(h.into()).unwrap(),
                        dkim_canonicalize_header_value(value, canonicalization).to_string(),
                    ))
                }
            }

            signed_headers.to_string()
        }
        DkimCanonicalizationType::Relaxed => {
            let mut signed_headers = String::new();

            for h in headers_list {
                let h = dkim_canonicalize_header_tag(h, canonicalization);
                if let Some(value) = mail_headers.get_raw(&h) {
                    write!(
                        signed_headers,
                        "{}:{}",
                        h,
                        dkim_canonicalize_header_value(value, canonicalization)
                    )
                    .expect("write implementation returned an error")
                }
            }

            signed_headers
        }
    }
}

/// Sign with Dkim a message by adding Dkim-Signture header created with configuration expressed by
/// dkim_config
pub(super) fn dkim_sign(message: &mut Message, dkim_config: &DkimConfig) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let headers = message.headers();
    let body_hash = Sha256::digest(&dkim_canonicalize_body(
        &message.body_raw(),
        dkim_config.canonicalization.body,
    ));
    let bh = base64::encode(body_hash);
    let mut signed_headers_list =
        dkim_config
            .headers
            .iter()
            .fold(String::new(), |mut list, header| {
                if !list.is_empty() {
                    list.push(':');
                }

                list.push_str(header);
                list
            });
    if let DkimCanonicalizationType::Relaxed = dkim_config.canonicalization.header {
        signed_headers_list.make_ascii_lowercase();
    }
    let dkim_header = dkim_header_format(dkim_config, timestamp, &signed_headers_list, &bh, "");
    let signed_headers = dkim_canonicalize_headers(
        dkim_config.headers.iter().map(|h| h.as_ref()),
        headers,
        dkim_config.canonicalization.header,
    );
    let canonicalized_dkim_header = dkim_canonicalize_headers(
        ["DKIM-Signature"],
        &dkim_header,
        dkim_config.canonicalization.header,
    );
    let mut hashed_headers = Sha256::new();
    hashed_headers.update(signed_headers.as_bytes());
    hashed_headers.update(canonicalized_dkim_header.trim_end().as_bytes());
    let hashed_headers = hashed_headers.finalize();
    let signature = match &dkim_config.private_key.0 {
        InnerDkimSigningKey::Rsa(private_key) => base64::encode(
            private_key
                .sign(
                    PaddingScheme::new_pkcs1v15_sign(Some(Hash::SHA2_256)),
                    &hashed_headers,
                )
                .unwrap(),
        ),
        InnerDkimSigningKey::Ed25519(private_key) => {
            base64::encode(private_key.sign(&hashed_headers).to_bytes())
        }
    };
    let dkim_header = dkim_header_format(
        dkim_config,
        timestamp,
        &signed_headers_list,
        &bh,
        &signature,
    );
    message.headers.insert_raw(HeaderValue::new(
        HeaderName::new_from_ascii_str("DKIM-Signature"),
        dkim_header.get_raw("DKIM-Signature").unwrap().to_owned(),
    ));
}

#[cfg(test)]
mod test {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };

    use super::{
        super::{
            header::{HeaderName, HeaderValue},
            Header, Message,
        },
        dkim_canonicalize_body, dkim_canonicalize_header_value, dkim_canonicalize_headers,
        DkimCanonicalizationType, DkimConfig, DkimSigningAlgorithm, DkimSigningKey,
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

        fn display(&self) -> HeaderValue {
            HeaderValue::new(Self::name(), self.0.clone())
        }
    }

    #[test]
    fn test_body_simple_canonicalize() {
        let body = b"test\r\n\r\ntest   \ttest\r\n\r\n\r\n";
        let expected: &[u8] = b"test\r\n\r\ntest   \ttest\r\n";
        assert_eq!(
            dkim_canonicalize_body(body, DkimCanonicalizationType::Simple),
            expected
        )
    }
    #[test]
    fn test_body_relaxed_canonicalize() {
        let body = b"test\r\n\r\ntest   \ttest\r\n\r\n\r\n";
        let expected: &[u8] = b"test\r\n\r\ntest test\r\n";
        assert_eq!(
            dkim_canonicalize_body(body, DkimCanonicalizationType::Relaxed),
            expected
        )
    }
    #[test]
    fn test_header_simple_canonicalize() {
        let value = "test\r\n\r\ntest   \ttest\r\n";
        let expected = "test\r\n\r\ntest   \ttest\r\n";
        assert_eq!(
            dkim_canonicalize_header_value(value, DkimCanonicalizationType::Simple),
            expected
        )
    }
    #[test]
    fn test_header_relaxed_canonicalize() {
        let value = "test\r\n\r\ntest   \ttest\r\n";
        let expected = "testtest test\r\n";
        assert_eq!(
            dkim_canonicalize_header_value(value, DkimCanonicalizationType::Relaxed),
            expected
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
        assert_eq!(dkim_canonicalize_headers(["From", "Test"], &message.headers, DkimCanonicalizationType::Simple),"From: Test <test+ezrz@example.net>\r\nTest: test  test very very long with spaces and extra spaces   \twill be \r\n folded to several lines \r\n")
    }
    #[test]
    fn test_headers_relaxed_canonicalize() {
        let message = test_message();
        assert_eq!(dkim_canonicalize_headers(["From", "Test"], &message.headers, DkimCanonicalizationType::Relaxed),"from:Test <test+ezrz@example.net>\r\ntest:test test very very long with spaces and extra spaces will be folded to several lines\r\n")
    }
    #[test]
    fn test_signature_rsa() {
        let mut message = test_message();
        let key = "-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAz+FHbM8BwkBBz/Ux5OYLQ5Bp1HVuCHTP6Rr3HXTnome/2cGl
/ze0tsmmFbCjjsS89MXbMGs9xJhjv18LmL1N0UTllblOizzVjorQyN4RwBOfG34j
7SS56pwzrA738Ry8FAbL5InPWEgVzbOhXuTCs8yuzcqTnm4sH/csnIl7cMWeQkVn
1FR9LKMtUG0fjhDPkdX0jx3qTX1L3Z7a7gX6geY191yNd9i9DvE2/+wMigMYz1LA
ts4alk2g86MQhtbjc8AOR7EC15hSw37/lmamlunYLa3wC+PzHNMA8sAfnmkgNvip
ssjh8LnelD9qn+VtsjQB5ppkeQx3TcUPvz5z+QIDAQABAoIBAQCzRa5ZEbSMlumq
s+PRaOox3CrIRHUd6c8bUlvmFVllX1++JRhInvvD3ubSMcD7cIMb/D1o5jMgheMP
uKHBmQ+w91+e3W30+gOZp/EiKRDZupIuHXxSGKgUwZx2N3pvfr5b7viLIKWllpTn
DpCNy251rIDbjGX97Tk0X+8jGBVSTCxtruGJR5a+hz4t9Z7bz7JjZWcRNJC+VA+Q
ATjnV7AHO1WR+0tAdPJaHsRLI7drKFSqTYq0As+MksZ40p7T6blZW8NUXA09fJRn
3mP2TZdWjjfBXZje026v4T7TZl+TELKw5WirL/UJ8Zw8dGGV6EZvbfMacZuUB1YQ
0vZnGe4BAoGBAO63xWP3OV8oLAMF90umuusPaQNSc6DnpjnP+sTAcXEYJA0Sa4YD
y8dpTAdFJ4YvUQhLxtbZFK5Ih3x7ZhuerLSJiZiDPC2IJJb7j/812zQQriOi4mQ8
bimxM4Nzql8FKGaXMppE5grFLsy8tw7neIM9KE4uwe9ajwJrRrOTUY8ZAoGBAN7t
+xFeuhg4F9expyaPpCvKT2YNAdMcDzpm7GtLX292u+DQgBfg50Ur9XmbS+RPlx1W
r2Sw3bTjRjJU9QnSZLL2w3hiii/wdaePI4SCaydHdLi4ZGz/pNUsUY+ck2pLptS0
F7rL+s9MV9lUyhvX+pIh+O3idMWAdaymzs7ZlgfhAoGAVoFn2Wrscmw3Tr0puVNp
JudFsbt+RU/Mr+SLRiNKuKX74nTLXBwiC1hAAd5wjTK2VaBIJPEzilikKFr7TIT6
ps20e/0KoKFWSRROQTh9/+cPg8Bx88rmTNt3BGq00Ywn8M1XvAm9pyd/Zxf36kG9
LSnLYlGVW6xgaIsBau+2vXkCgYAeChVdxtTutIhJ8U9ju9FUcUN3reMEDnDi3sGW
x6ZJf8dbSN0p2o1vXbgLNejpD+x98JNbzxVg7Ysk9xu5whb9opC+ZRDX2uAPvxL7
JRPJTDCnP3mQ0nXkn78xydh3Z1BIsyfLbPcT/eaMi4dcbyL9lARWEcDIaEHzDNsr
NlioIQKBgQCXIZp5IBfG5WSXzFk8xvP4BUwHKEI5bttClBmm32K+vaSz8qO6ak6G
4frg+WVopFg3HBHdK9aotzPEd0eHMXJv3C06Ynt2lvF+Rgi/kwGbkuq/mFVnmYYR
Fz0TZ6sKrTAF3fdkN3bcQv6JG1CfnWENDGtekemwcCEA9v46/RsOfg==
-----END RSA PRIVATE KEY-----";
        let signing_key = DkimSigningKey::new(key.to_string(), DkimSigningAlgorithm::Rsa).unwrap();
        message.sign(&DkimConfig::default_config(
            "dkimtest".to_string(),
            "example.org".to_string(),
            signing_key,
        ));
        println!("{}", std::str::from_utf8(&message.formatted()).unwrap());
        let mut verify_command = Command::new("dkimverify")
            .stdin(Stdio::piped())
            .spawn()
            .expect("Fail to verify message signature");
        let mut stdin = verify_command.stdin.take().expect("Failed to open stdin");
        std::thread::spawn(move || {
            stdin
                .write_all(&message.formatted())
                .expect("Failed to write to stdin");
        });
        assert!(verify_command
            .wait()
            .expect("Command did not run")
            .success());
    }
}
