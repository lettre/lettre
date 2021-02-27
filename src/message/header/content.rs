use hyperx::{
    header::{Formatter as HeaderFormatter, Header, RawLike},
    Error as HeaderError, Result as HyperResult,
};
use std::{
    fmt::{Display, Formatter as FmtFormatter, Result as FmtResult},
    str::{from_utf8, FromStr},
};

header! {
    /// `Content-Id` header, defined in [RFC2045](https://tools.ietf.org/html/rfc2045#section-7)
    (ContentId, "Content-ID") => [String]
}

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
    fn header_name() -> &'static str {
        "Content-Transfer-Encoding"
    }

    // FIXME HeaderError->HeaderError, same for result
    fn parse_header<'a, T>(raw: &'a T) -> HyperResult<Self>
    where
        T: RawLike<'a>,
        Self: Sized,
    {
        raw.one()
            .ok_or(HeaderError::Header)
            .and_then(|r| from_utf8(r).map_err(|_| HeaderError::Header))
            .and_then(|s| {
                s.parse::<ContentTransferEncoding>()
                    .map_err(|_| HeaderError::Header)
            })
    }

    fn fmt_header(&self, f: &mut HeaderFormatter<'_, '_>) -> FmtResult {
        f.fmt_line(&format!("{}", self))
    }
}

#[cfg(test)]
mod test {
    use super::ContentTransferEncoding;
    use hyperx::header::Headers;

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
