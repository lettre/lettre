use hyperx::{
    header::{Formatter as HeaderFormatter, Header, RawLike},
    Error as HeaderError, Result as HyperResult,
};
use std::{
    fmt::{Display, Formatter as FmtFormatter, Result as FmtResult},
    str::{from_utf8, FromStr},
};

header! { (ContentId, "Content-ID") => [String] }

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
        ContentTransferEncoding::SevenBit
    }
}

impl Display for ContentTransferEncoding {
    fn fmt(&self, f: &mut FmtFormatter<'_>) -> FmtResult {
        use self::ContentTransferEncoding::*;
        f.write_str(match *self {
            SevenBit => "7bit",
            QuotedPrintable => "quoted-printable",
            Base64 => "base64",
            EightBit => "8bit",
            Binary => "binary",
        })
    }
}

impl FromStr for ContentTransferEncoding {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::ContentTransferEncoding::*;
        match s {
            "7bit" => Ok(SevenBit),
            "quoted-printable" => Ok(QuotedPrintable),
            "base64" => Ok(Base64),
            "8bit" => Ok(EightBit),
            "binary" => Ok(Binary),
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
