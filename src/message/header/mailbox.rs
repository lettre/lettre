use super::{Header, HeaderName};
use crate::message::mailbox::{Mailbox, Mailboxes};
use crate::BoxError;

/// Header which can contains multiple mailboxes
pub trait MailboxesHeader {
    fn join_mailboxes(&mut self, other: Self);
}

macro_rules! mailbox_header {
    ($(#[$doc:meta])*($type: ident, $name: expr)) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $type(Mailbox);

        impl Header for $type {
            fn name() -> HeaderName {
                HeaderName::new_from_ascii_static($name)
            }

            fn parse_value(s: &str) -> Result<Self,BoxError> {
                Ok(Self(s.parse()?))
            }

            fn display(&self) -> String {
                self.0.to_string()
            }
        }

        impl std::convert::From<Mailbox> for $type {
            #[inline]
            fn from(mailbox: Mailbox) -> Self {
                Self(mailbox)
            }
        }

        impl std::convert::From<$type> for Mailbox {
            #[inline]
            fn from(this: $type) -> Mailbox {
                this.0
            }
        }
    };
}

macro_rules! mailboxes_header {
    ($(#[$doc:meta])*($type: ident, $name: expr)) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $type(pub(crate) Mailboxes);

        impl MailboxesHeader for $type {
            fn join_mailboxes(&mut self, other: Self) {
                self.0.extend(other.0);
            }
        }

        impl Header for $type {
            fn name() -> HeaderName {
                HeaderName::new_from_ascii_static($name)
            }

            fn parse_value(s: &str) -> Result<Self, BoxError> {
                Ok(Self(s.parse()?))
            }

            fn display(&self) -> String {
                self.0.to_string()
            }
        }

        impl std::convert::From<Mailboxes> for $type {
            #[inline]
            fn from(mailboxes: Mailboxes) -> Self {
                Self(mailboxes)
            }
        }

        impl std::convert::From<$type> for Mailboxes {
            #[inline]
            fn from(this: $type) -> Mailboxes {
                this.0
            }
        }
    };
}

mailbox_header! {
    /**

    `Sender` header

    This header contains [`Mailbox`][self::Mailbox] associated with sender.

    ```no_test
    header::Sender("Mr. Sender <sender@example.com>".parse().unwrap())
    ```
     */
    (Sender, "Sender")
}

mailboxes_header! {
    /**

    `From` header

    This header contains [`Mailboxes`][self::Mailboxes].

     */
    (From, "From")
}

mailboxes_header! {
    /**

    `Reply-To` header

    This header contains [`Mailboxes`][self::Mailboxes].

     */
    (ReplyTo, "Reply-To")
}

mailboxes_header! {
    /**

    `To` header

    This header contains [`Mailboxes`][self::Mailboxes].

     */
    (To, "To")
}

mailboxes_header! {
    /**

    `Cc` header

    This header contains [`Mailboxes`][self::Mailboxes].

     */
    (Cc, "Cc")
}

mailboxes_header! {
    /**

    `Bcc` header

    This header contains [`Mailboxes`][self::Mailboxes].

     */
    (Bcc, "Bcc")
}

/*
fn parse_mailboxes(raw: &[u8]) -> HyperResult<Mailboxes> {
    if let Ok(src) = from_utf8(raw) {
        if let Ok(mbs) = src.parse() {
            return Ok(mbs);
        }
    }
    Err(HeaderError::Header)
}

fn format_mailboxes<'a>(mbs: Iter<'a, Mailbox>, f: &mut HeaderFormatter<'_, '_>) -> FmtResult {
    f.fmt_line(&Mailboxes::from(
        mbs.map(|mb| mb.recode_name(utf8_b::encode))
            .collect::<Vec<_>>(),
    ))
}
*/

#[cfg(test)]
mod test {
    use super::{From, Mailbox, Mailboxes};
    use crate::message::header::{HeaderName, Headers};

    #[test]
    fn format_single_without_name() {
        let from = Mailboxes::new().with("kayo@example.com".parse().unwrap());

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(format!("{}", headers), "From: kayo@example.com\r\n");
    }

    #[test]
    fn format_single_with_name() {
        let from = Mailboxes::new().with("K. <kayo@example.com>".parse().unwrap());

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(format!("{}", headers), "From: K. <kayo@example.com>\r\n");
    }

    #[test]
    fn format_multi_without_name() {
        let from = Mailboxes::new()
            .with("kayo@example.com".parse().unwrap())
            .with("pony@domain.tld".parse().unwrap());

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(
            format!("{}", headers),
            "From: kayo@example.com, pony@domain.tld\r\n"
        );
    }

    #[test]
    fn format_multi_with_name() {
        let from = vec![
            "K. <kayo@example.com>".parse().unwrap(),
            "Pony P. <pony@domain.tld>".parse().unwrap(),
        ];

        let mut headers = Headers::new();
        headers.set(From(from.into()));

        assert_eq!(
            format!("{}", headers),
            "From: K. <kayo@example.com>, Pony P. <pony@domain.tld>\r\n"
        );
    }

    #[test]
    fn format_single_with_utf8_name() {
        let from = vec!["Кайо <kayo@example.com>".parse().unwrap()];

        let mut headers = Headers::new();
        headers.set(From(from.into()));

        assert_eq!(
            headers.to_string(),
            "From: =?utf-8?b?0JrQsNC50L4=?= <kayo@example.com>\r\n"
        );
    }

    #[test]
    fn parse_single_without_name() {
        let from = vec!["kayo@example.com".parse().unwrap()].into();

        let mut headers = Headers::new();
        headers.set_raw(
            HeaderName::new_from_ascii_static("From"),
            "kayo@example.com".into(),
        );

        assert_eq!(headers.get::<From>(), Some(From(from)));
    }

    #[test]
    fn parse_single_with_name() {
        let from = vec!["K. <kayo@example.com>".parse().unwrap()].into();

        let mut headers = Headers::new();
        headers.set_raw(
            HeaderName::new_from_ascii_static("From"),
            "K. <kayo@example.com>".into(),
        );

        assert_eq!(headers.get::<From>(), Some(From(from)));
    }

    #[test]
    fn parse_multi_without_name() {
        let from: Vec<Mailbox> = vec![
            "kayo@example.com".parse().unwrap(),
            "pony@domain.tld".parse().unwrap(),
        ];

        let mut headers = Headers::new();
        headers.set_raw(
            HeaderName::new_from_ascii_static("From"),
            "kayo@example.com, pony@domain.tld".into(),
        );

        assert_eq!(headers.get::<From>(), Some(From(from.into())));
    }

    #[test]
    fn parse_multi_with_name() {
        let from: Vec<Mailbox> = vec![
            "K. <kayo@example.com>".parse().unwrap(),
            "Pony P. <pony@domain.tld>".parse().unwrap(),
        ];

        let mut headers = Headers::new();
        headers.set_raw(
            HeaderName::new_from_ascii_static("From"),
            "K. <kayo@example.com>, Pony P. <pony@domain.tld>".into(),
        );

        assert_eq!(headers.get::<From>(), Some(From(from.into())));
    }

    #[test]
    fn parse_single_with_utf8_name() {
        let from: Vec<Mailbox> = vec!["Кайо <kayo@example.com>".parse().unwrap()];

        let mut headers = Headers::new();
        headers.set_raw(
            HeaderName::new_from_ascii_static("From"),
            "=?utf-8?b?0JrQsNC50L4=?= <kayo@example.com>".into(),
        );

        assert_eq!(headers.get::<From>(), Some(From(from.into())));
    }
}
