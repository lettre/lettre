use std::{
    error::Error as StdError,
    fmt::{self, Display},
    str::FromStr,
};

use mime::Mime;

use super::{Header, HeaderName};
use crate::BoxError;

/// `Content-Type` of the body
///
/// Defined in [RFC2045](https://tools.ietf.org/html/rfc2045#section-5)
#[derive(Debug, Clone, PartialEq)]
pub struct ContentType(Mime);

impl ContentType {
    /// A `ContentType` of type `text/plain; charset=utf-8`
    ///
    /// Indicates that the body is in utf-8 encoded plain text.
    pub const TEXT_PLAIN: ContentType = Self::from_mime(mime::TEXT_PLAIN_UTF_8);

    /// A `ContentType` of type `text/html; charset=utf-8`
    ///
    /// Indicates that the body is in utf-8 encoded html.
    pub const TEXT_HTML: ContentType = Self::from_mime(mime::TEXT_HTML_UTF_8);

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
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("Content-Type")
    }

    fn parse(s: &str) -> Result<Self, BoxError> {
        Ok(Self(s.parse()?))
    }

    fn display(&self) -> String {
        self.0.to_string()
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
    use super::ContentType;
    use crate::message::header::Headers;

    #[test]
    fn format_content_type() {
        let mut headers = Headers::new();

        headers.set(ContentType::TEXT_PLAIN);

        assert_eq!(
            format!("{}", headers),
            "Content-Type: text/plain; charset=utf-8\r\n"
        );

        headers.set(ContentType::TEXT_HTML);

        assert_eq!(
            format!("{}", headers),
            "Content-Type: text/html; charset=utf-8\r\n"
        );
    }

    #[test]
    fn parse_content_type() {
        let mut headers = Headers::new();

        headers.set_raw("Content-Type", "text/plain; charset=utf-8");

        assert_eq!(headers.get::<ContentType>(), Some(&ContentType::TEXT_PLAIN));

        headers.set_raw("Content-Type", "text/html; charset=utf-8");

        assert_eq!(headers.get::<ContentType>(), Some(&ContentType::TEXT_HTML));
    }
}
