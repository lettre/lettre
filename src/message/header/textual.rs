use super::{Header, HeaderName};
use crate::BoxError;

macro_rules! text_header {
    ($(#[$attr:meta])* Header($type_name: ident, $header_name: expr )) => {
        $(#[$attr])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $type_name(String);

        impl Header for $type_name {
            fn name() -> HeaderName {
                HeaderName::new_from_ascii_str($header_name)
            }

            fn parse(s: &str) -> Result<Self, BoxError> {
                Ok(Self(s.into()))
            }

            fn display(&self) -> String {
                self.0.clone()
            }
        }

        impl From<String> for $type_name {
            #[inline]
            fn from(text: String) -> Self {
                Self(text)
            }
        }

        impl AsRef<str> for $type_name {
            #[inline]
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

text_header!(
    /// `Comments` of the message, defined in [RFC5322](https://tools.ietf.org/html/rfc5322#section-3.6.5)
    Header(Comments, "Comments")
);
text_header!(
    /// `Keywords` header. Should contain a comma-separated list of one or more
    /// words or quoted-strings, defined in [RFC5322](https://tools.ietf.org/html/rfc5322#section-3.6.5)
    Header(Keywords, "Keywords")
);
text_header!(
    /// `In-Reply-To` header. Contains one or more
    /// unique message identifiers,
    /// defined in [RFC5322](https://tools.ietf.org/html/rfc5322#section-3.6.4)
    Header(InReplyTo, "In-Reply-To")
);
text_header!(
    /// `References` header. Contains one or more
    /// unique message identifiers,
    /// defined in [RFC5322](https://tools.ietf.org/html/rfc5322#section-3.6.4)
    Header(References, "References")
);
text_header!(
    /// `Message-Id` header. Contains a unique message identifier,
    /// defined in [RFC5322](https://tools.ietf.org/html/rfc5322#section-3.6.4)
    Header(MessageId, "Message-ID")
);
text_header!(
    /// `User-Agent` header. Contains information about the client,
    /// defined in [draft-melnikov-email-user-agent-00](https://tools.ietf.org/html/draft-melnikov-email-user-agent-00#section-3)
    Header(UserAgent, "User-Agent")
);
text_header! {
    /// `Content-Id` header,
    /// defined in [RFC2045](https://tools.ietf.org/html/rfc2045#section-7)
    Header(ContentId, "Content-ID")
}
text_header! {
    /// `Content-Location` header,
    /// defined in [RFC2110](https://tools.ietf.org/html/rfc2110#section-4.3)
    Header(ContentLocation, "Content-Location")
}

#[cfg(test)]
mod test {
    use super::Subject;
    use crate::message::header::{HeaderName, Headers};

    #[test]
    fn format_ascii() {
        let mut headers = Headers::new();
        headers.set(Subject("Sample subject".into()));

        assert_eq!(headers.to_string(), "Subject: Sample subject\r\n");
    }

    #[test]
    fn format_utf8() {
        let mut headers = Headers::new();
        headers.set(Subject("Тема сообщения".into()));

        assert_eq!(
            headers.to_string(),
            "Subject: =?utf-8?b?0KLQtdC80LAg0YHQvtC+0LHRidC10L3QuNGP?=\r\n"
        );
    }

    #[test]
    fn parse_ascii() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("Subject"),
            "Sample subject".to_string(),
        );

        assert_eq!(
            headers.get::<Subject>(),
            Some(Subject("Sample subject".into()))
        );
    }
}
