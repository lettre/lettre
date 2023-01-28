use std::fmt::Write;

use email_encoding::headers::EmailWriter;

use super::{Header, HeaderName, HeaderValue};
use crate::BoxError;

/// `Content-Disposition` of an attachment
///
/// Defined in [RFC2183](https://tools.ietf.org/html/rfc2183)
#[derive(Debug, Clone, PartialEq)]
pub struct ContentDisposition(HeaderValue);

impl ContentDisposition {
    /// An attachment which should be displayed inline into the message
    pub fn inline() -> Self {
        Self(HeaderValue::dangerous_new_pre_encoded(
            Self::name(),
            "inline".to_string(),
            "inline".to_string(),
        ))
    }

    /// An attachment which should be displayed inline into the message, but that also
    /// species the filename in case it were to be downloaded
    pub fn inline_with_name(file_name: &str) -> Self {
        Self::with_name("inline", file_name)
    }

    /// An attachment which is separate from the body of the message, and can be downloaded separately
    pub fn attachment(file_name: &str) -> Self {
        Self::with_name("attachment", file_name)
    }

    fn with_name(kind: &str, file_name: &str) -> Self {
        let raw_value = format!("{kind}; filename=\"{file_name}\"");

        let mut encoded_value = String::new();
        let line_len = "Content-Disposition: ".len();
        {
            let mut w = EmailWriter::new(&mut encoded_value, line_len, 0, false, false);
            w.write_str(kind).expect("writing `kind` returned an error");
            w.write_char(';').expect("writing `;` returned an error");
            w.optional_breakpoint();

            email_encoding::headers::rfc2231::encode("filename", file_name, &mut w)
                .expect("some Write implementation returned an error");
        }

        Self(HeaderValue::dangerous_new_pre_encoded(
            Self::name(),
            raw_value,
            encoded_value,
        ))
    }
}

impl Header for ContentDisposition {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("Content-Disposition")
    }

    fn parse(s: &str) -> Result<Self, BoxError> {
        match (s.split_once(';'), s) {
            (_, "inline") => Ok(Self::inline()),
            (Some((kind @ ("inline" | "attachment"), file_name)), _) => file_name
                .split_once(" filename=\"")
                .and_then(|(_, file_name)| file_name.strip_suffix('"'))
                .map(|file_name| Self::with_name(kind, file_name))
                .ok_or_else(|| "Unsupported ContentDisposition value".into()),
            _ => Err("Unsupported ContentDisposition value".into()),
        }
    }

    fn display(&self) -> HeaderValue {
        self.0.clone()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

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
