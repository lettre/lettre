//! Provides a strongly typed way to build emails
//!
//! ## Usage
//!
//! This section demonstrates how to build messages.
//!
//! <!--
//! style for <details><summary>Blablabla</summary> Lots of stuff</details>
//! borrowed from https://docs.rs/time/0.2.23/src/time/lib.rs.html#49-54
//! -->
//! <style>
//! summary, details:not([open]) { cursor: pointer; }
//! summary { display: list-item; }
//! summary::marker { content: '▶ '; }
//! details[open] summary::marker { content: '▼ '; }
//! </style>
//!
//!
//! ### Plain body
//!
//! The easiest way of creating a message, which uses a plain text body.
//!
//! ```rust
//! use lettre::message::Message;
//!
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let m = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body(String::from("Be happy!"))?;
//! # Ok(())
//! # }
//! ```
//!
//! Which produces:
//! <details>
//! <summary>Click to expand</summary>
//!
//! ```sh
//! From: NoBody <nobody@domain.tld>
//! Reply-To: Yuin <yuin@domain.tld>
//! To: Hei <hei@domain.tld>
//! Subject: Happy new year
//! Date: Sat, 12 Dec 2020 16:33:19 GMT
//! Content-Transfer-Encoding: 7bit
//!
//! Be happy!
//! ```
//! </details>
//! <br />
//!
//! The unicode header data is encoded using _UTF8-Base64_ encoding, when necessary.
//!
//! The `Content-Transfer-Encoding` is chosen based on the best encoding
//! available for the given body, between `7bit`, `quoted-printable` and `base64`.
//!
//! ### Plain and HTML body
//!
//! Uses a MIME body to include both plain text and HTML versions of the body.
//!
//! ```rust
//! # use std::error::Error;
//! use lettre::message::{header, Message, MultiPart, Part, SinglePart};
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let m = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .multipart(
//!         MultiPart::alternative()
//!             .singlepart(
//!                 SinglePart::builder()
//!                     .header(header::ContentType("text/plain; charset=utf8".parse()?))
//!                     .body(String::from("Hello, world! :)")),
//!             )
//!             .singlepart(
//!                 SinglePart::builder()
//!                     .header(header::ContentType("text/html; charset=utf8".parse()?))
//!                     .body(String::from(
//!                         "<p><b>Hello</b>, <i>world</i>! <img src=\"cid:123\"></p>",
//!                     )),
//!             ),
//!     )?;
//! # Ok(())
//! # }
//! ```
//!
//! Which produces:
//! <details>
//! <summary>Click to expand</summary>
//!
//! ```sh
//! From: NoBody <nobody@domain.tld>
//! Reply-To: Yuin <yuin@domain.tld>
//! To: Hei <hei@domain.tld>
//! Subject: Happy new year
//! MIME-Version: 1.0
//! Date: Sat, 12 Dec 2020 16:33:19 GMT
//! Content-Type: multipart/alternative; boundary="0oVZ2r6AoLAhLlb0gPNSKy6BEqdS2IfwxrcbUuo1"
//!
//! --0oVZ2r6AoLAhLlb0gPNSKy6BEqdS2IfwxrcbUuo1
//! Content-Type: text/plain; charset=utf8
//! Content-Transfer-Encoding: 7bit
//!
//! Hello, world! :)
//! --0oVZ2r6AoLAhLlb0gPNSKy6BEqdS2IfwxrcbUuo1
//! Content-Type: text/html; charset=utf8
//! Content-Transfer-Encoding: 7bit
//!
//! <p><b>Hello</b>, <i>world</i>! <img src="cid:123"></p>
//! --0oVZ2r6AoLAhLlb0gPNSKy6BEqdS2IfwxrcbUuo1--
//! ```
//! </details>
//!
//! ### Complex MIME body
//!
//! This example shows how to include both plain and HTML versions of the body,
//! attachments and inlined images.
//!
//! ```rust
//! # use std::error::Error;
//! use lettre::message::{header, Body, Message, MultiPart, Part, SinglePart};
//! use std::fs;
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let image = fs::read("docs/lettre.png")?;
//! // this image_body can be cloned and reused between emails.
//! // since `Body` holds a pre-encoded body, reusing it means avoiding having
//! // to re-encode the same body for every email (this clearly only applies
//! // when sending multiple emails with the same attachment).
//! let image_body = Body::new(image);
//!
//! let m = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .multipart(
//!         MultiPart::mixed()
//!             .multipart(
//!                 MultiPart::alternative()
//!                     .singlepart(
//!                         SinglePart::builder()
//!                             .header(header::ContentType("text/plain; charset=utf8".parse()?))
//!                             .body(String::from("Hello, world! :)")),
//!                     )
//!                     .multipart(
//!                         MultiPart::related()
//!                             .singlepart(
//!                                 SinglePart::builder()
//!                                     .header(header::ContentType(
//!                                         "text/html; charset=utf8".parse()?,
//!                                     ))
//!                                     .body(String::from(
//!                                         "<p><b>Hello</b>, <i>world</i>! <img src=cid:123></p>",
//!                                     )),
//!                             )
//!                             .singlepart(
//!                                 SinglePart::builder()
//!                                     .header(header::ContentType("image/png".parse()?))
//!                                     .header(header::ContentDisposition::inline())
//!                                     .header(header::ContentId::from(String::from("<123>")))
//!                                     .body(image_body),
//!                             ),
//!                     ),
//!             )
//!             .singlepart(
//!                 SinglePart::builder()
//!                     .header(header::ContentType("text/plain; charset=utf8".parse()?))
//!                     .header(header::ContentDisposition::attachment("example.rs"))
//!                     .body(String::from("fn main() { println!(\"Hello, World!\") }")),
//!             ),
//!     )?;
//! # Ok(())
//! # }
//! ```
//!
//! Which produces:
//! <details>
//! <summary>Click to expand</summary>
//!
//! ```sh
//! From: NoBody <nobody@domain.tld>
//! Reply-To: Yuin <yuin@domain.tld>
//! To: Hei <hei@domain.tld>
//! Subject: Happy new year
//! MIME-Version: 1.0
//! Date: Sat, 12 Dec 2020 16:30:45 GMT
//! Content-Type: multipart/mixed; boundary="0oVZ2r6AoLAhLlb0gPNSKy6BEqdS2IfwxrcbUuo1"
//!
//! --0oVZ2r6AoLAhLlb0gPNSKy6BEqdS2IfwxrcbUuo1
//! Content-Type: multipart/alternative; boundary="EyXdAZIgZuyUjAounq4Aj44a6MpJfqCKhm6pE1zk"
//!
//! --EyXdAZIgZuyUjAounq4Aj44a6MpJfqCKhm6pE1zk
//! Content-Type: text/plain; charset=utf8
//! Content-Transfer-Encoding: 7bit
//!
//! Hello, world! :)
//! --EyXdAZIgZuyUjAounq4Aj44a6MpJfqCKhm6pE1zk
//! Content-Type: multipart/related; boundary="eM5Z18WZVOQsqi5GQ71XGAXk6NNvHUA1Xv1FWrXr"
//!
//! --eM5Z18WZVOQsqi5GQ71XGAXk6NNvHUA1Xv1FWrXr
//! Content-Type: text/html; charset=utf8
//! Content-Transfer-Encoding: 7bit
//!
//! <p><b>Hello</b>, <i>world</i>! <img src=cid:123></p>
//! --eM5Z18WZVOQsqi5GQ71XGAXk6NNvHUA1Xv1FWrXr
//! Content-Type: image/png
//! Content-Disposition: inline
//! Content-ID: <123>
//! Content-Transfer-Encoding: base64
//!
//! PHNtaWxlLXJhdy1pbWFnZS1kYXRhPg==
//! --eM5Z18WZVOQsqi5GQ71XGAXk6NNvHUA1Xv1FWrXr--
//! --EyXdAZIgZuyUjAounq4Aj44a6MpJfqCKhm6pE1zk--
//! --0oVZ2r6AoLAhLlb0gPNSKy6BEqdS2IfwxrcbUuo1
//! Content-Type: text/plain; charset=utf8
//! Content-Disposition: attachment; filename="example.rs"
//! Content-Transfer-Encoding: 7bit
//!
//! fn main() { println!("Hello, World!") }
//! --0oVZ2r6AoLAhLlb0gPNSKy6BEqdS2IfwxrcbUuo1--
//! ```
//! </details>

pub use body::{Body, IntoBody, MaybeString};
pub use mailbox::*;
pub use mimebody::*;

pub use mime;

mod body;
pub mod header;
mod mailbox;
mod mimebody;
mod utf8_b;

use std::{convert::TryFrom, io::Write, time::SystemTime};

use uuid::Uuid;

use crate::{
    address::Envelope,
    message::header::{ContentTransferEncoding, EmailDate, Header, Headers, MailboxesHeader},
    Error as EmailError,
};

const DEFAULT_MESSAGE_ID_DOMAIN: &str = "localhost";

/// Something that can be formatted as an email message
trait EmailFormat {
    // Use a writer?
    fn format(&self, out: &mut Vec<u8>);
}

/// A builder for messages
#[derive(Debug, Clone)]
pub struct MessageBuilder {
    headers: Headers,
    envelope: Option<Envelope>,
}

impl MessageBuilder {
    /// Creates a new default message builder
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
            envelope: None,
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
        let s: String = subject.into();
        self.header(header::Subject::from(s))
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
    /// Defined in [RFC5322](https://tools.ietf.org/html/rfc5322#section-3.6.2).
    ///
    /// Shortcut for `self.header(header::Sender(mbox))`.
    pub fn sender(self, mbox: Mailbox) -> Self {
        self.header(header::Sender::from(mbox))
    }

    /// Set or add mailbox to `From` header
    ///
    /// Defined in [RFC5322](https://tools.ietf.org/html/rfc5322#section-3.6.2).
    ///
    /// Shortcut for `self.mailbox(header::From(mbox))`.
    pub fn from(self, mbox: Mailbox) -> Self {
        self.mailbox(header::From::from(Mailboxes::from(mbox)))
    }

    /// Set or add mailbox to `ReplyTo` header
    ///
    /// Defined in [RFC5322](https://tools.ietf.org/html/rfc5322#section-3.6.2).
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
        self.header(header::InReplyTo::from(id))
    }

    /// Set or add message id to [`References`
    /// header](https://tools.ietf.org/html/rfc5322#section-3.6.4)
    pub fn references(self, id: String) -> Self {
        self.header(header::References::from(id))
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
            Some(i) => self.header(header::MessageId::from(i)),
            None => {
                #[cfg(feature = "hostname")]
                let hostname = hostname::get()
                    .map_err(|_| ())
                    .and_then(|s| s.into_string().map_err(|_| ()))
                    .unwrap_or_else(|_| DEFAULT_MESSAGE_ID_DOMAIN.to_string());
                #[cfg(not(feature = "hostname"))]
                let hostname = DEFAULT_MESSAGE_ID_DOMAIN.to_string();

                self.header(header::MessageId::from(
                    // https://tools.ietf.org/html/rfc5322#section-3.6.4
                    format!("<{}@{}>", Uuid::new_v4(), hostname),
                ))
            }
        }
    }

    /// Set [User-Agent
    /// header](https://tools.ietf.org/html/draft-melnikov-email-user-agent-004)
    pub fn user_agent(self, id: String) -> Self {
        self.header(header::UserAgent::from(id))
    }

    /// Force specific envelope (by default it is derived from headers)
    pub fn envelope(mut self, envelope: Envelope) -> Self {
        self.envelope = Some(envelope);
        self
    }

    // TODO: High-level methods for attachments and embedded files

    /// Create message from body
    fn build(self, body: MessageBody) -> Result<Message, EmailError> {
        // Check for missing required headers
        // https://tools.ietf.org/html/rfc5322#section-3.6

        // Insert Date if missing
        let res = if self.headers.get::<header::Date>().is_none() {
            self.date_now()
        } else {
            self
        };

        // Fail is missing correct originator (Sender or From)
        match res.headers.get::<header::From>() {
            Some(header::From(f)) => {
                let from: Vec<Mailbox> = f.clone().into();
                if from.len() > 1 && res.headers.get::<header::Sender>().is_none() {
                    return Err(EmailError::TooManyFrom);
                }
            }
            None => {
                return Err(EmailError::MissingFrom);
            }
        }

        let envelope = match res.envelope {
            Some(e) => e,
            None => Envelope::try_from(&res.headers)?,
        };
        Ok(Message {
            headers: res.headers,
            body,
            envelope,
        })
    }

    /// Create [`Message`] using a [`Vec<u8>`], [`String`], or [`Body`] body
    ///
    /// Automatically gets encoded with `7bit`, `quoted-printable` or `base64`
    /// `Content-Transfer-Encoding`, based on the most efficient and valid encoding
    /// for `body`.
    pub fn body<T: IntoBody>(mut self, body: T) -> Result<Message, EmailError> {
        let maybe_encoding = self.headers.get::<ContentTransferEncoding>().copied();
        let body = body.into_body(maybe_encoding);

        self.headers.set(body.encoding());
        self.build(MessageBody::Raw(body.into_vec()))
    }

    /// Create message using mime body ([`MultiPart`][self::MultiPart])
    pub fn multipart(self, part: MultiPart) -> Result<Message, EmailError> {
        self.mime_1_0().build(MessageBody::Mime(Part::Multi(part)))
    }

    /// Create message using mime body ([`SinglePart`][self::SinglePart])
    pub fn singlepart(self, part: SinglePart) -> Result<Message, EmailError> {
        self.mime_1_0().build(MessageBody::Mime(Part::Single(part)))
    }
}

/// Email message which can be formatted
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
#[derive(Clone, Debug)]
pub struct Message {
    headers: Headers,
    body: MessageBody,
    envelope: Envelope,
}

#[derive(Clone, Debug)]
enum MessageBody {
    Mime(Part),
    Raw(Vec<u8>),
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
        write!(out, "{}", self.headers)
            .expect("A Write implementation panicked while formatting headers");

        match &self.body {
            MessageBody::Mime(p) => p.format(out),
            MessageBody::Raw(r) => {
                out.extend_from_slice(b"\r\n");
                out.extend_from_slice(&r)
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
    use crate::message::{header, mailbox::Mailbox, Message, MultiPart, SinglePart};

    #[test]
    fn email_missing_originator() {
        assert!(Message::builder()
            .body(String::from("Happy new year!"))
            .is_err());
    }

    #[test]
    fn email_miminal_message() {
        assert!(Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .to("NoBody <nobody@domain.tld>".parse().unwrap())
            .body(String::from("Happy new year!"))
            .is_ok());
    }

    #[test]
    fn email_missing_sender() {
        assert!(Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .from("AnyBody <anybody@domain.tld>".parse().unwrap())
            .body(String::from("Happy new year!"))
            .is_err());
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
            .header(header::Subject::from(String::from("яңа ел белән!")))
            .body(String::from("Happy new year!"))
            .unwrap();

        assert_eq!(
            String::from_utf8(email.formatted()).unwrap(),
            concat!(
                "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
                "From: =?utf-8?b?0JrQsNC4?= <kayo@example.com>\r\n",
                "To: Pony O.P. <pony@domain.tld>\r\n",
                "Subject: =?utf-8?b?0Y/So9CwINC10Lsg0LHQtdC705nQvSE=?=\r\n",
                "Content-Transfer-Encoding: 7bit\r\n",
                "\r\n",
                "Happy new year!"
            )
        );
    }

    #[test]
    fn email_with_png() {
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();
        let img = std::fs::read("./docs/lettre.png").unwrap();
        let m = Message::builder()
            .date(date)
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .multipart(
                MultiPart::related()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType(
                                "text/html; charset=utf8".parse().unwrap(),
                            ))
                            .body(String::from(
                                "<p><b>Hello</b>, <i>world</i>! <img src=cid:123></p>",
                            )),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType("image/png".parse().unwrap()))
                            .header(header::ContentDisposition::inline())
                            .header(header::ContentId::from(String::from("<123>")))
                            .body(img),
                    ),
            )
            .unwrap();

        let output = String::from_utf8(m.formatted()).unwrap();
        let file_expected = std::fs::read("./testdata/email_with_png.eml").unwrap();
        let expected = String::from_utf8(file_expected).unwrap();

        for (i, line) in output.lines().zip(expected.lines()).enumerate() {
            if i == 6 || i == 8 || i == 13 || i == 232 {
                continue;
            }

            assert_eq!(line.0, line.1)
        }
    }
}
