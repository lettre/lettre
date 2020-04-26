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
use std::{convert::TryFrom, time::SystemTime};
use uuid::Uuid;

const DEFAULT_MESSAGE_ID_DOMAIN: &str = "localhost";

pub trait EmailFormat {
    // Use a writer?
    fn format(&self, out: &mut Vec<u8>);
}

/// A builder for messages
#[derive(Debug, Clone)]
pub struct MessageBuilder {
    headers: Headers,
}

impl MessageBuilder {
    /// Creates a new default message builder
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
        }
    }

    /// Set custom header to message
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
    pub fn date(self, date: EmailDate) -> Self {
        self.header(header::Date(date))
    }

    /// Set `Date` header using current date/time
    ///
    /// Shortcut for `self.date(SystemTime::now())`.
    pub fn date_now(self) -> Self {
        self.date(SystemTime::now().into())
    }

    /// Set `Subject` header to message
    ///
    /// Shortcut for `self.header(header::Subject(subject.into()))`.
    pub fn subject<S: Into<String>>(self, subject: S) -> Self {
        self.header(header::Subject(subject.into()))
    }

    /// Set `Mime-Version` header to 1.0
    ///
    /// Shortcut for `self.header(header::MIME_VERSION_1_0)`.
    ///
    /// Not exposed as it is set by body methods
    fn mime_1_0(self) -> Self {
        self.header(header::MIME_VERSION_1_0)
    }

    /// Set `Sender` header. Should be used when providing several `From` mailboxes.
    ///
    /// https://tools.ietf.org/html/rfc5322#section-3.6.2
    ///
    /// Shortcut for `self.header(header::Sender(mbox))`.
    pub fn sender(self, mbox: Mailbox) -> Self {
        self.header(header::Sender(mbox))
    }

    /// Set or add mailbox to `From` header
    ///
    /// https://tools.ietf.org/html/rfc5322#section-3.6.2
    ///
    /// Shortcut for `self.mailbox(header::From(mbox))`.
    pub fn from(self, mbox: Mailbox) -> Self {
        self.mailbox(header::From(mbox.into()))
    }

    /// Set or add mailbox to `ReplyTo` header
    ///
    /// https://tools.ietf.org/html/rfc5322#section-3.6.2
    ///
    /// Shortcut for `self.mailbox(header::ReplyTo(mbox))`.
    pub fn reply_to(self, mbox: Mailbox) -> Self {
        self.mailbox(header::ReplyTo(mbox.into()))
    }

    /// Set or add mailbox to `To` header
    ///
    /// Shortcut for `self.mailbox(header::To(mbox))`.
    pub fn to(self, mbox: Mailbox) -> Self {
        self.mailbox(header::To(mbox.into()))
    }

    /// Set or add mailbox to `Cc` header
    ///
    /// Shortcut for `self.mailbox(header::Cc(mbox))`.
    pub fn cc(self, mbox: Mailbox) -> Self {
        self.mailbox(header::Cc(mbox.into()))
    }

    /// Set or add mailbox to `Bcc` header
    ///
    /// Shortcut for `self.mailbox(header::Bcc(mbox))`.
    pub fn bcc(self, mbox: Mailbox) -> Self {
        self.mailbox(header::Bcc(mbox.into()))
    }

    /// Set or add message id to [`In-Reply-To`
    /// header](https://tools.ietf.org/html/rfc5322#section-3.6.4)
    pub fn in_reply_to(self, id: String) -> Self {
        self.header(header::InReplyTo(id))
    }

    /// Set or add message id to [`References`
    /// header](https://tools.ietf.org/html/rfc5322#section-3.6.4)
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
    pub fn user_agent(self, id: String) -> Self {
        self.header(header::UserAgent(id))
    }

    fn insert_missing_headers(self) -> Self {
        let mut new = self;

        // Insert Date if missing
        new = if new.headers.get::<header::Date>().is_none() {
            new.date_now()
        } else {
            new
        };

        // TODO insert sender if needed?
        new
    }

    // TODO: High-level methods for attachments and embedded files

    /// Create message from body
    fn build(self, body: Body) -> Result<Message, EmailError> {
        let res = self.insert_missing_headers();

        let envelope = Envelope::try_from(&res.headers)?;
        Ok(Message {
            headers: res.headers,
            body,
            envelope,
        })
    }

    // In theory having a body is optional

    /// Plain ASCII body
    ///
    /// *WARNING*: Generally not what you want
    pub fn body<T: Into<String>>(self, body: T) -> Result<Message, EmailError> {
        // 998 chars by line
        // CR and LF MUST only occur together as CRLF; they MUST NOT appear
        //  independently in the body.
        let body = body.into();

        if !&body.is_ascii() {
            return Err(EmailError::NonAsciiChars);
        }

        self.build(Body::Raw(body))
    }

    /// Create message using mime body ([`MultiPart`](::MultiPart))
    pub fn multipart(self, part: MultiPart) -> Result<Message, EmailError> {
        self.mime_1_0().build(Body::Mime(Part::Multi(part)))
    }

    /// Create message using mime body ([`SinglePart`](::SinglePart)
    pub fn singlepart(self, part: SinglePart) -> Result<Message, EmailError> {
        self.mime_1_0().build(Body::Mime(Part::Single(part)))
    }
}

/// Email message which can be formatted
#[derive(Clone, Debug)]
pub struct Message {
    headers: Headers,
    body: Body,
    envelope: Envelope,
}

#[derive(Clone, Debug)]
enum Body {
    Mime(Part),
    Raw(String),
}

impl Message {
    /// Create a new message builder without headers
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    /// Get the headers from the Message
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get `Message` envelope
    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }

    /// Get message content formatted for SMTP
    pub fn formatted(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.format(&mut out);
        out
    }
}

impl EmailFormat for Message {
    fn format(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.headers.to_string().as_bytes());
        match &self.body {
            Body::Mime(p) => p.format(out),
            Body::Raw(r) => {
                out.extend_from_slice(b"\r\n");
                out.extend(r.as_bytes())
            }
        }
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        MessageBuilder::new()
    }
}

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
            String::from_utf8(email.formatted()).unwrap(),
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
