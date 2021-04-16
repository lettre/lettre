use std::{
    fmt::{Display, Formatter as FmtFormatter, Result as FmtResult},
    str::FromStr,
};

use super::{Header, HeaderName};
use crate::BoxError;

/// `Content-Transfer-Encoding` of the body
///
/// The `Message` builder takes care of choosing the most
/// efficient encoding based on the chosen body, so in most
/// use-caches this header shouldn't be set manually.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentTransferEncoding {
    SevenBit,
    QuotedPrintable,
    Base64,
    // 8BITMIME
    EightBit,
    Binary,
}

impl Header for ContentTransferEncoding {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("Content-Transfer-Encoding")
    }

    fn parse(s: &str) -> Result<Self, BoxError> {
        Ok(s.parse()?)
    }

    fn display(&self) -> String {
        self.to_string()
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
    use super::ContentTransferEncoding;
    use crate::message::header::Headers;

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

        headers.set_raw("Content-Transfer-Encoding", "7bit");

        assert_eq!(
            headers.get::<ContentTransferEncoding>(),
            Some(&ContentTransferEncoding::SevenBit)
        );

        headers.set_raw("Content-Transfer-Encoding", "base64");

        assert_eq!(
            headers.get::<ContentTransferEncoding>(),
            Some(&ContentTransferEncoding::Base64)
        );
    }
}
