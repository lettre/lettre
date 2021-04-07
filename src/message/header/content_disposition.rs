use std::{fmt::Result as FmtResult, str::from_utf8};

use hyperx::{
    header::{Formatter as HeaderFormatter, Header, RawLike},
    Error as HeaderError, Result as HyperResult,
};

/// `Content-Disposition` of an attachment
///
/// Defined in [RFC2183](https://tools.ietf.org/html/rfc2183)
#[derive(Debug, Clone, PartialEq)]
pub struct ContentDisposition(String);

impl ContentDisposition {
    /// An attachment which should be displayed inline into the message
    pub fn inline() -> Self {
        Self("inline".into())
    }

    /// An attachment which should be displayed inline into the message, but that also
    /// species the filename in case it were to be downloaded
    pub fn inline_with_name(file_name: &str) -> Self {
        debug_assert!(!file_name.contains('"'), "file_name shouldn't contain '\"'");
        Self(format!("inline; filename=\"{}\"", file_name))
    }

    /// An attachment which is separate from the body of the message, and can be downloaded separately
    pub fn attachment(file_name: &str) -> Self {
        debug_assert!(!file_name.contains('"'), "file_name shouldn't contain '\"'");
        Self(format!("attachment; filename=\"{}\"", file_name))
    }
}

impl Header for ContentDisposition {
    fn header_name() -> &'static str {
        "Content-Disposition"
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
            .map(|s| Self(s.into()))
    }

    fn fmt_header(&self, f: &mut HeaderFormatter<'_, '_>) -> FmtResult {
        f.fmt_line(&self.0)
    }
}

#[cfg(test)]
mod test {
    use super::ContentDisposition;
    use hyperx::header::Headers;

    #[test]
    fn format_content_disposition() {
        let mut headers = Headers::new();

        headers.set(ContentDisposition::inline());

        assert_eq!(format!("{}", headers), "Content-Disposition: inline\r\n");

        headers.set(ContentDisposition::attachment("something.txt"));

        assert_eq!(
            format!("{}", headers),
            "Content-Disposition: attachment; filename=\"something.txt\"\r\n"
        );
    }

    #[test]
    fn parse_content_disposition() {
        let mut headers = Headers::new();

        headers.set_raw("Content-Disposition", "inline");

        assert_eq!(
            headers.get::<ContentDisposition>(),
            Some(&ContentDisposition::inline())
        );

        headers.set_raw(
            "Content-Disposition",
            "attachment; filename=\"something.txt\"",
        );

        assert_eq!(
            headers.get::<ContentDisposition>(),
            Some(&ContentDisposition::attachment("something.txt"))
        );
    }
}
