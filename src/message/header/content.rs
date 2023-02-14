use std::{
    fmt::{Display, Formatter as FmtFormatter, Result as FmtResult},
    str::FromStr,
};

use super::{Header, HeaderName, HeaderValue};
use crate::BoxError;

/// `Content-Transfer-Encoding` of the body
///
/// The `Message` builder takes care of choosing the most
/// efficient encoding based on the chosen body, so in most
/// use-caches this header shouldn't be set manually.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContentTransferEncoding {
    /// ASCII
    SevenBit,
    /// Quoted-Printable encoding
    QuotedPrintable,
    /// base64 encoding
    Base64,
    /// Requires `8BITMIME`
    EightBit,
    /// Binary data
    Binary,
}

impl Header for ContentTransferEncoding {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("Content-Transfer-Encoding")
    }

    fn parse(s: &str) -> Result<Self, BoxError> {
        Ok(s.parse()?)
    }

    fn display(&self) -> HeaderValue {
        let val = self.to_string();
        HeaderValue::dangerous_new_pre_encoded(Self::name(), val.clone(), val)
    }
}

impl Display for ContentTransferEncoding {
    fn fmt(&self, f: &mut FmtFormatter<'_>) -> FmtResult {
        f.write_str(match *self {
            Self::SevenBit => "7bit",
            Self::QuotedPrintable => "quoted-printable",
            Self::Base64 => "base64",
            Self::EightBit => "8bit",
            Self::Binary => "binary",
        })
    }
}

impl FromStr for ContentTransferEncoding {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "7bit" => Ok(Self::SevenBit),
            "quoted-printable" => Ok(Self::QuotedPrintable),
            "base64" => Ok(Self::Base64),
            "8bit" => Ok(Self::EightBit),
            "binary" => Ok(Self::Binary),
            _ => Err(s.into()),
        }
    }
}

impl Default for ContentTransferEncoding {
    fn default() -> Self {
        ContentTransferEncoding::Base64
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::ContentTransferEncoding;
    use crate::message::header::{HeaderName, HeaderValue, Headers};

    #[test]
    fn format_content_transfer_encoding() {
        let mut headers = Headers::new();

        headers.set(ContentTransferEncoding::SevenBit);

        assert_eq!(headers.to_string(), "Content-Transfer-Encoding: 7bit\r\n");

        headers.set(ContentTransferEncoding::Base64);

        assert_eq!(headers.to_string(), "Content-Transfer-Encoding: base64\r\n");
    }

    #[test]
    fn parse_content_transfer_encoding() {
        let mut headers = Headers::new();

        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("Content-Transfer-Encoding"),
            "7bit".to_owned(),
        ));

        assert_eq!(
            headers.get::<ContentTransferEncoding>(),
            Some(ContentTransferEncoding::SevenBit)
        );

        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("Content-Transfer-Encoding"),
            "base64".to_owned(),
        ));

        assert_eq!(
            headers.get::<ContentTransferEncoding>(),
            Some(ContentTransferEncoding::Base64)
        );
    }
}
