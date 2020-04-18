//! Provides a strongly typed way to build emails

pub use encoder::*;
pub use mailbox::*;
pub use mimebody::*;

pub use mime;

mod encoder;
pub mod header;
mod mailbox;
mod mimebody;
mod utf8_b;

use crate::{
    message::header::{EmailDate, Header, Headers, MailboxesHeader},
    Envelope, Error as EmailError,
};
use bytes::Bytes;
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
    time::SystemTime,
};
use uuid::Uuid;

const DEFAULT_MESSAGE_ID_DOMAIN: &str = "localhost";

/// A builder for messages
#[derive(Debug, Clone)]
pub struct MessageBuilder {
    headers: Headers,
}

impl MessageBuilder {
    /// Creates a new default message builder
    #[inline]
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
        }
    }

    /// Set custom header to message
    #[inline]
    pub fn header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Add mailbox to header
    pub fn mailbox<H: Header + MailboxesHeader>(mut self, header: H) -> Self {
        if self.headers.has::<H>() {
            self.headers.get_mut::<H>().unwrap().join_mailboxes(header);
            self
        } else {
            self.header(header)
        }
    }

    /// Add `Date` header to message
    ///
    /// Shortcut for `self.header(header::Date(date))`.
    #[inline]
    pub fn date(self, date: EmailDate) -> Self {
        self.header(header::Date(date))
    }

    /// Set `Date` header using current date/time
    ///
    /// Shortcut for `self.date(SystemTime::now())`.
    #[inline]
    pub fn date_now(self) -> Self {
        self.date(SystemTime::now().into())
    }

    /// Set `Subject` header to message
    ///
    /// Shortcut for `self.header(header::Subject(subject.into()))`.
    #[inline]
    pub fn subject<S: Into<String>>(self, subject: S) -> Self {
        self.header(header::Subject(subject.into()))
    }

    /// Set `Mime-Version` header to 1.0
    ///
    /// Shortcut for `self.header(header::MIME_VERSION_1_0)`.
    #[inline]
    pub fn mime_1_0(self) -> Self {
        self.header(header::MIME_VERSION_1_0)
    }

    /// Set `Sender` header. Should be used when providing several `From` mailboxes.
    ///
    /// https://tools.ietf.org/html/rfc5322#section-3.6.2
    ///
    /// Shortcut for `self.header(header::Sender(mbox))`.
    #[inline]
    pub fn sender(self, mbox: Mailbox) -> Self {
        self.header(header::Sender(mbox))
    }

    /// Set or add mailbox to `From` header
    ///
    /// https://tools.ietf.org/html/rfc5322#section-3.6.2
    ///
    /// Shortcut for `self.mailbox(header::From(mbox))`.
    #[inline]
    pub fn from(self, mbox: Mailbox) -> Self {
        self.mailbox(header::From(mbox.into()))
    }

    /// Set or add mailbox to `ReplyTo` header
    ///
    /// https://tools.ietf.org/html/rfc5322#section-3.6.2
    ///
    /// Shortcut for `self.mailbox(header::ReplyTo(mbox))`.
    #[inline]
    pub fn reply_to(self, mbox: Mailbox) -> Self {
        self.mailbox(header::ReplyTo(mbox.into()))
    }

    /// Set or add mailbox to `To` header
    ///
    /// Shortcut for `self.mailbox(header::To(mbox))`.
    #[inline]
    pub fn to(self, mbox: Mailbox) -> Self {
        self.mailbox(header::To(mbox.into()))
    }

    /// Set or add mailbox to `Cc` header
    ///
    /// Shortcut for `self.mailbox(header::Cc(mbox))`.
    #[inline]
    pub fn cc(self, mbox: Mailbox) -> Self {
        self.mailbox(header::Cc(mbox.into()))
    }

    /// Set or add mailbox to `Bcc` header
    ///
    /// Shortcut for `self.mailbox(header::Bcc(mbox))`.
    #[inline]
    pub fn bcc(self, mbox: Mailbox) -> Self {
        self.mailbox(header::Bcc(mbox.into()))
    }

    /// Set or add message id to [`In-Reply-To`
    /// header](https://tools.ietf.org/html/rfc5322#section-3.6.4)
    #[inline]
    pub fn in_reply_to(self, id: String) -> Self {
        self.header(header::InReplyTo(id))
    }

    /// Set or add message id to [`References`
    /// header](https://tools.ietf.org/html/rfc5322#section-3.6.4)
    #[inline]
    pub fn references(self, id: String) -> Self {
        self.header(header::References(id))
    }

    /// Set [Message-Id
    /// header](https://tools.ietf.org/html/rfc5322#section-3.6.4)
    ///
    /// Should generally be inserted by the mail relay.
    ///
    /// If `None` is provided, an id will be generated in the
    /// `<UUID@HOSTNAME>`.
    #[inline]
    pub fn message_id(self, id: Option<String>) -> Self {
        match id {
            Some(i) => self.header(header::MessageId(i)),
            None => {
                #[cfg(feature = "hostname")]
                let hostname = hostname::get()
                    .map_err(|_| ())
                    .and_then(|s| s.into_string().map_err(|_| ()))
                    .unwrap_or_else(|_| DEFAULT_MESSAGE_ID_DOMAIN.to_string());
                #[cfg(not(feature = "hostname"))]
                let hostname = DEFAULT_MESSAGE_ID_DOMAIN.to_string();

                self.header(header::MessageId(
                    // https://tools.ietf.org/html/rfc5322#section-3.6.4
                    format!("<{}@{}>", Uuid::new_v4(), hostname),
                ))
            }
        }
    }

    /// Set [User-Agent
    /// header](https://tools.ietf.org/html/draft-melnikov-email-user-agent-004)
    #[inline]
    pub fn user_agent(self, id: String) -> Self {
        self.header(header::UserAgent(id))
    }

    fn insert_missing_headers(self) -> Self {
        // Insert Date if missing
        if self.headers.get::<header::Date>().is_none() {
            self.date_now()
        } else {
            self
        }
        // TODO insert sender if needed?
    }

    // TODO: High-level methods for attachments and embedded files

    /// Create message by joining content
    #[inline]
    fn build<T>(self, body: T, split: bool) -> Result<Message<T>, EmailError> {
        let res = self.insert_missing_headers();
        let envelope = Envelope::try_from(&res.headers)?;
        Ok(Message {
            headers: res.headers,
            split,
            body,
            envelope,
        })
    }

    /// Create message using body
    #[inline]
    pub fn body<T>(self, body: T) -> Result<Message<T>, EmailError> {
        self.build(body, true)
    }

    /// Create message using mime body ([`MultiPart`](::MultiPart) or [`SinglePart`](::SinglePart))
    // FIXME restrict usage on MIME?
    #[inline]
    pub fn mime_body<T>(self, body: T) -> Result<Message<T>, EmailError> {
        self.mime_1_0().build(body, false)
    }
}

/// Email message which can be formatted
#[derive(Clone, Debug)]
pub struct Message<B = Bytes> {
    headers: Headers,
    split: bool,
    body: B,
    envelope: Envelope,
}

impl Message<()> {
    /// Create a new message builder without headers
    #[inline]
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }
}

impl<B> Message<B> {
    /// Get the headers from the Message
    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Read the body
    #[inline]
    pub fn body_ref(&self) -> &B {
        &self.body
    }

    /// Try to extract envelope data from `Message` headers
    #[inline]
    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        MessageBuilder::new()
    }
}

impl<B> Display for Message<B>
where
    B: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.headers.fmt(f)?;
        if self.split {
            f.write_str("\r\n")?;
        }
        self.body.fmt(f)
    }
}

// An email is Message + Envelope

#[cfg(test)]
mod test {
    use crate::message::{header, mailbox::Mailbox, Message};

    #[test]
    fn email_message() {
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();

        let email = Message::builder()
            .date(date)
            .header(header::From(
                vec![Mailbox::new(
                    Some("Каи".into()),
                    "kayo@example.com".parse().unwrap(),
                )]
                .into(),
            ))
            .header(header::To(
                vec!["Pony O.P. <pony@domain.tld>".parse().unwrap()].into(),
            ))
            .header(header::Subject("яңа ел белән!".into()))
            .body("Happy new year!")
            .unwrap();

        assert_eq!(
            format!("{}", email),
            concat!(
                "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
                "From: =?utf-8?b?0JrQsNC4?= <kayo@example.com>\r\n",
                "To: Pony O.P. <pony@domain.tld>\r\n",
                "Subject: =?utf-8?b?0Y/So9CwINC10Lsg0LHQtdC705nQvSE=?=\r\n",
                "\r\n",
                "Happy new year!"
            )
        );
    }
}
