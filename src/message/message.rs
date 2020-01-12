use crate::message::header::{self, EmailDate, Header, Headers, MailboxesHeader};
use crate::message::Mailbox;
use crate::{Envelope, Error as EmailError};
use bytes::Bytes;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::SystemTime;

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

    /// Add `Date:` header to message
    ///
    /// Shortcut for `self.header(header::Date(date))`.
    #[inline]
    pub fn date(self, date: EmailDate) -> Self {
        self.header(header::Date(date))
    }

    /// Set `Date:` header using current date/time
    ///
    /// Shortcut for `self.date(SystemTime::now())`.
    #[inline]
    pub fn date_now(self) -> Self {
        self.date(SystemTime::now().into())
    }

    /// Set `Subject:` header to message
    ///
    /// Shortcut for `self.header(header::Subject(subject.into()))`.
    #[inline]
    pub fn subject<S: Into<String>>(self, subject: S) -> Self {
        self.header(header::Subject(subject.into()))
    }

    /// Set `Mime-Version:` header to 1.0
    ///
    /// Shortcut for `self.header(header::MIME_VERSION_1_0)`.
    #[inline]
    pub fn mime_1_0(self) -> Self {
        self.header(header::MIME_VERSION_1_0)
    }

    /// Set `Sender:` header
    ///
    /// Shortcut for `self.header(header::Sender(mbox))`.
    #[inline]
    pub fn sender(self, mbox: Mailbox) -> Self {
        self.header(header::Sender(mbox))
    }

    /// Set or add mailbox to `From:` header
    ///
    /// Shortcut for `self.mailbox(header::From(mbox))`.
    #[inline]
    pub fn from(self, mbox: Mailbox) -> Self {
        self.mailbox(header::From(mbox.into()))
    }

    /// Set or add mailbox to `ReplyTo:` header
    ///
    /// Shortcut for `self.mailbox(header::ReplyTo(mbox))`.
    #[inline]
    pub fn reply_to(self, mbox: Mailbox) -> Self {
        self.mailbox(header::ReplyTo(mbox.into()))
    }

    /// Set or add mailbox to `To:` header
    ///
    /// Shortcut for `self.mailbox(header::To(mbox))`.
    #[inline]
    pub fn to(self, mbox: Mailbox) -> Self {
        self.mailbox(header::To(mbox.into()))
    }

    /// Set or add mailbox to `Cc:` header
    ///
    /// Shortcut for `self.mailbox(header::Cc(mbox))`.
    #[inline]
    pub fn cc(self, mbox: Mailbox) -> Self {
        self.mailbox(header::Cc(mbox.into()))
    }

    /// Set or add mailbox to `Bcc:` header
    ///
    /// Shortcut for `self.mailbox(header::Bcc(mbox))`.
    #[inline]
    pub fn bcc(self, mbox: Mailbox) -> Self {
        self.mailbox(header::Bcc(mbox.into()))
    }

    // FIXME we need to:
    //
    // * add shortcuts for attachment support
    // * add shortcuts for embedded images

    // Add a build() method (optional, also allow raw message) to:
    //
    // * check the validity of our headers
    // * by default use TextNone for message-id
    // * insert missing ones (date, message-id, sender, etc.)
    // * extract an envelope (add en envelope builder in the MessageBuilder probably)

    /// Create message using body
    #[inline]
    pub fn body<T>(self, body: T) -> Message<T> {
        Message {
            headers: self.headers,
            split: true,
            body,
        }
    }

    /// Create message by joining content
    #[inline]
    pub fn join<T>(self, body: T) -> Message<T> {
        Message {
            headers: self.headers,
            split: false,
            body,
        }
    }

    /// Create message using mime body ([`MultiPart`](::MultiPart) or [`SinglePart`](::SinglePart))
    ///
    /// Shortcut for `self.mime_1_0().join(body)`.
    #[inline]
    pub fn mime_body<T>(self, body: T) -> Message<T> {
        self.mime_1_0().join(body)
    }

    /// Try to extract envelope data from `Message` headers
    pub fn envelope(&self) -> Result<Envelope, EmailError> {
        Envelope::try_from(&self.headers)
    }
}

/// Email message which can be formatted or streamed
#[derive(Clone, Debug)]
pub struct Message<B = Bytes> {
    headers: Headers,
    split: bool,
    body: B,
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

    /// Get a mutable reference to the headers
    #[inline]
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Set the body
    #[inline]
    pub fn set_body<T: Into<B>>(&mut self, body: T) {
        self.body = body.into();
    }

    /// Read the body
    #[inline]
    pub fn body_ref(&self) -> &B {
        &self.body
    }

    /// Try to extract envelope data from `Message` headers
    pub fn envelope(&self) -> Result<Envelope, EmailError> {
        Envelope::try_from(&self.headers)
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

#[cfg(test)]
mod test {
    use crate::message::header;
    use crate::message::mailbox::Mailbox;
    use crate::message::message::Message;

    #[test]
    fn date_header() {
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();

        let email = Message::builder().date(date).body("");

        assert_eq!(
            format!("{}", email),
            "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n\r\n"
        );
    }

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
            .body("Happy new year!");

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
