use super::{Header, HeaderName, HeaderValue};
use crate::BoxError;

macro_rules! text_header {
    ($(#[$attr:meta])* Header($type_name: ident, $header_name: expr )) => {
        $(#[$attr])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $type_name(String);

        impl Header for $type_name {
            fn name() -> HeaderName {
                HeaderName::new_from_ascii_str($header_name)
            }

            fn parse(s: &str) -> Result<Self, BoxError> {
                Ok(Self(s.into()))
            }

            fn display(&self) -> HeaderValue {
             HeaderValue::new(Self::name(),   self.0.clone())
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
    /// `Subject` of the message, defined in [RFC5322](https://tools.ietf.org/html/rfc5322#section-3.6.5)
    Header(Subject, "Subject")
);
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
text_header! {
    /// `List-Id` header. Identifies the mailing list a message belongs to,
    /// defined in [RFC2919](https://tools.ietf.org/html/rfc2919#section-3)
    Header(ListId, "List-Id")
}
text_header! {
    /// `List-Help` header. Contains one or more `<URL>`s pointing at help
    /// for the list, defined in [RFC2369](https://tools.ietf.org/html/rfc2369#section-3.1)
    Header(ListHelp, "List-Help")
}
text_header! {
    /// `List-Unsubscribe` header. Contains one or more `<URL>`s for removing
    /// oneself from the list, defined in [RFC2369](https://tools.ietf.org/html/rfc2369#section-3.2)
    Header(ListUnsubscribe, "List-Unsubscribe")
}
text_header! {
    /// `List-Subscribe` header. Contains one or more `<URL>`s for adding
    /// oneself to the list, defined in [RFC2369](https://tools.ietf.org/html/rfc2369#section-3.3)
    Header(ListSubscribe, "List-Subscribe")
}
text_header! {
    /// `List-Post` header. Contains one or more `<URL>`s for posting to the
    /// list, defined in [RFC2369](https://tools.ietf.org/html/rfc2369#section-3.4)
    Header(ListPost, "List-Post")
}
text_header! {
    /// `List-Owner` header. Contains one or more `<URL>`s for contacting the
    /// list owner, defined in [RFC2369](https://tools.ietf.org/html/rfc2369#section-3.5)
    Header(ListOwner, "List-Owner")
}
text_header! {
    /// `List-Archive` header. Contains one or more `<URL>`s pointing at an
    /// archive of the list, defined in [RFC2369](https://tools.ietf.org/html/rfc2369#section-3.6)
    Header(ListArchive, "List-Archive")
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::{ListId, ListUnsubscribe, Subject};
    use crate::message::header::{HeaderName, HeaderValue, Headers};

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
    fn format_utf8_word() {
        let mut headers = Headers::new();
        headers.set(Subject("Administratör".into()));

        assert_eq!(
            headers.to_string(),
            "Subject: =?utf-8?b?QWRtaW5pc3RyYXTDtnI=?=\r\n"
        );
    }

    #[test]
    fn parse_ascii() {
        let mut headers = Headers::new();
        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("Subject"),
            "Sample subject".to_owned(),
        ));

        assert_eq!(
            headers.get::<Subject>(),
            Some(Subject("Sample subject".into()))
        );
    }

    #[test]
    fn format_list_headers() {
        let mut headers = Headers::new();
        headers.set(ListId("Example <list.example.com>".into()));
        headers.set(ListUnsubscribe(
            "<mailto:unsub@example.com>, <https://example.com/unsub>".into(),
        ));

        assert_eq!(
            headers.to_string(),
            "List-Id: Example <list.example.com>\r\n\
             List-Unsubscribe: <mailto:unsub@example.com>, <https://example.com/unsub>\r\n"
        );
    }

    #[test]
    fn parse_list_id() {
        let mut headers = Headers::new();
        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("List-Id"),
            "Example <list.example.com>".to_owned(),
        ));

        assert_eq!(
            headers.get::<ListId>(),
            Some(ListId("Example <list.example.com>".into()))
        );
    }
}
