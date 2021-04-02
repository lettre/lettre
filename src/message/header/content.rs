use mime::Mime;

use super::{Header, HeaderName};
use crate::BoxError;
use std::{
    fmt::{Display, Formatter as FmtFormatter, Result as FmtResult},
    str::FromStr,
};

#[derive(Clone)]
pub struct ContentType(Mime);

impl Header for ContentType {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_static("Content-Type")
    }

    fn parse_value(s: &str) -> Result<Self, BoxError> {
        Ok(Self(s.parse()?))
    }

    fn display(&self) -> String {
        self.0.to_string()
    }
}

impl From<Mime> for ContentType {
    #[inline]
    fn from(m: Mime) -> Self {
        Self(m)
    }
}

impl From<ContentType> for Mime {
    #[inline]
    fn from(this: ContentType) -> Mime {
        this.0
    }
}

/// `Content-Transfer-Encoding` of the body
///
/// The `Message` builder takes care of choosing the most
/// efficient encoding based on the chosen body, so in most
/// use-caches this header shouldn't be set manually.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ContentTransferEncoding {
    SevenBit,
    QuotedPrintable,
    Base64,
    // 8BITMIME
    EightBit,
    Binary,
}

impl Default for ContentTransferEncoding {
    fn default() -> Self {
        ContentTransferEncoding::Base64
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

impl Header for ContentTransferEncoding {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_static("Content-Transfer-Encoding")
    }

    fn parse_value(s: &str) -> Result<Self, BoxError> {
        Ok(s.parse()?)
    }

    fn display(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::ContentTransferEncoding;
    use crate::message::header::{HeaderName, Headers};

    #[test]
    fn format_content_transfer_encoding() {
        let mut headers = Headers::new();

        headers.set(ContentTransferEncoding::SevenBit);

        assert_eq!(
            format!("{}", headers),
            "Content-Transfer-Encoding: 7bit\r\n"
        );

        headers.set(ContentTransferEncoding::Base64);

        assert_eq!(
            format!("{}", headers),
            "Content-Transfer-Encoding: base64\r\n"
        );
    }

    #[test]
    fn parse_content_transfer_encoding() {
        let mut headers = Headers::new();

        headers.set_raw(
            HeaderName::new_from_ascii_static("Content-Transfer-Encoding"),
            "7bit".into(),
        );

        assert_eq!(
            headers.get::<ContentTransferEncoding>(),
            Some(ContentTransferEncoding::SevenBit)
        );

        headers.set_raw(
            HeaderName::new_from_ascii_static("Content-Transfer-Encoding"),
            "base64".into(),
        );

        assert_eq!(
            headers.get::<ContentTransferEncoding>(),
            Some(ContentTransferEncoding::Base64)
        );
    }
}
