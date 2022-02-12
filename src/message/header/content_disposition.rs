use super::{Header, HeaderName, HeaderValue};
use crate::BoxError;

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
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("Content-Disposition")
    }

    fn parse(s: &str) -> Result<Self, BoxError> {
        Ok(Self(s.into()))
    }

    fn display(&self) -> HeaderValue {
        HeaderValue::new(Self::name(), self.0.clone())
    }
}

#[cfg(test)]
mod test {
    use super::ContentDisposition;
    use crate::message::header::{HeaderName, HeaderValue, Headers};

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

        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("Content-Disposition"),
            "inline".to_string(),
        ));

        assert_eq!(
            headers.get::<ContentDisposition>(),
            Some(ContentDisposition::inline())
        );

        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("Content-Disposition"),
            "attachment; filename=\"something.txt\"".to_string(),
        ));

        assert_eq!(
            headers.get::<ContentDisposition>(),
            Some(ContentDisposition::attachment("something.txt"))
        );
    }
}
