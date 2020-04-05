use crate::message::utf8_b;
use hyperx::{
    header::{Formatter as HeaderFormatter, Header, RawLike},
    Error as HyperError, Result as HyperResult,
};
use std::{fmt::Result as FmtResult, str::from_utf8};

macro_rules! text_header {
    ( $type_name: ident, $header_name: expr ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $type_name(pub String);

        impl Header for $type_name {
            fn header_name() -> &'static str {
                $header_name
            }

            fn parse_header<'a, T>(raw: &'a T) -> HyperResult<$type_name>
            where
                T: RawLike<'a>,
                Self: Sized,
            {
                raw.one()
                    .ok_or(HyperError::Header)
                    .and_then(parse_text)
                    .map($type_name)
            }

            fn fmt_header(&self, f: &mut HeaderFormatter) -> FmtResult {
                fmt_text(&self.0, f)
            }
        }
    };
}

text_header!(Subject, "Subject");
text_header!(Comments, "Comments");
text_header!(Keywords, "Keywords");

fn parse_text(raw: &[u8]) -> HyperResult<String> {
    if let Ok(src) = from_utf8(raw) {
        if let Some(txt) = utf8_b::decode(src) {
            return Ok(txt);
        }
    }
    Err(HyperError::Header)
}

fn fmt_text(s: &str, f: &mut HeaderFormatter) -> FmtResult {
    f.fmt_line(&utf8_b::encode(s))
}

#[cfg(test)]
mod test {
    use super::Subject;
    use hyperx::header::Headers;

    #[test]
    fn format_ascii() {
        let mut headers = Headers::new();
        headers.set(Subject("Sample subject".into()));

        assert_eq!(format!("{}", headers), "Subject: Sample subject\r\n");
    }

    #[test]
    fn format_utf8() {
        let mut headers = Headers::new();
        headers.set(Subject("Тема сообщения".into()));

        assert_eq!(
            format!("{}", headers),
            "Subject: =?utf-8?b?0KLQtdC80LAg0YHQvtC+0LHRidC10L3QuNGP?=\r\n"
        );
    }

    #[test]
    fn parse_ascii() {
        let mut headers = Headers::new();
        headers.set_raw("Subject", "Sample subject");

        assert_eq!(
            headers.get::<Subject>(),
            Some(&Subject("Sample subject".into()))
        );
    }

    #[test]
    fn parse_utf8() {
        let mut headers = Headers::new();
        headers.set_raw(
            "Subject",
            "=?utf-8?b?0KLQtdC80LAg0YHQvtC+0LHRidC10L3QuNGP?=",
        );

        assert_eq!(
            headers.get::<Subject>(),
            Some(&Subject("Тема сообщения".into()))
        );
    }
}
