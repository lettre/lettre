use std::{
    error::Error as StdError,
    fmt::{self, Display, Result as FmtResult},
    str::{from_utf8, FromStr},
};

use hyperx::{
    header::{Formatter as HeaderFormatter, Header, RawLike},
    Error as HeaderError, Result as HyperResult,
};
use mime::Mime;

/// `Content-Type` of the body
///
/// Defined in [RFC2045](https://tools.ietf.org/html/rfc2045#section-5)
#[derive(Debug, Clone, PartialEq)]
pub struct ContentType(Mime);

impl ContentType {
    /// A `ContentType` of type `text/plain; charset=utf-8`
    pub const PLAIN_STRING: ContentType = Self::from_mime(mime::TEXT_PLAIN_UTF_8);

    /// A `ContentType` of type `text/html; charset=utf-8`
    pub const HTML_STRING: ContentType = Self::from_mime(mime::TEXT_HTML_UTF_8);

    /// Parse `s` into `ContentType`
    pub fn parse(s: &str) -> Result<ContentType, ContentTypeErr> {
        Ok(Self::from_mime(s.parse().map_err(ContentTypeErr)?))
    }

    pub(crate) const fn from_mime(mime: Mime) -> Self {
        Self(mime)
    }

    pub(crate) fn as_ref(&self) -> &Mime {
        &self.0
    }
}

impl Header for ContentType {
    fn header_name() -> &'static str {
        "Content-Type"
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
            .and_then(|s| s.parse::<Mime>().map(Self).map_err(|_| HeaderError::Header))
    }

    fn fmt_header(&self, f: &mut HeaderFormatter<'_, '_>) -> FmtResult {
        f.fmt_line(&self.0)
    }
}

impl FromStr for ContentType {
    type Err = ContentTypeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// An error occurred while trying to [`ContentType::parse`].
#[derive(Debug)]
pub struct ContentTypeErr(mime::FromStrError);

impl StdError for ContentTypeErr {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.0)
    }
}

impl Display for ContentTypeErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod test {
    use hyperx::header::Headers;

    use super::ContentType;

    #[test]
    fn format_content_type() {
        let mut headers = Headers::new();

        headers.set(ContentType::PLAIN_STRING);

        assert_eq!(
            format!("{}", headers),
            "Content-Type: text/plain; charset=utf-8\r\n"
        );

        headers.set(ContentType::HTML_STRING);

        assert_eq!(
            format!("{}", headers),
            "Content-Type: text/html; charset=utf-8\r\n"
        );
    }

    #[test]
    fn parse_content_type() {
        let mut headers = Headers::new();

        headers.set_raw("Content-Type", "text/plain; charset=utf-8");

        assert_eq!(
            headers.get::<ContentType>(),
            Some(&ContentType::PLAIN_STRING)
        );

        headers.set_raw("Content-Type", "text/html; charset=utf-8");

        assert_eq!(
            headers.get::<ContentType>(),
            Some(&ContentType::HTML_STRING)
        );
    }
}
