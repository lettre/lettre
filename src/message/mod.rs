//! Provides a strongly typed way to build emails
//!
//! ## Usage
//!
//! This section demonstrates how to build messages.
//!
//! <style>
//! summary, details:not([open]) { cursor: pointer; }
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
//! use lettre::message::{header, Message, MultiPart, SinglePart};
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let m = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .multipart(MultiPart::alternative_plain_html(
//!         String::from("Hello, world! :)"),
//!         String::from("<p><b>Hello</b>, <i>world</i>! <img src=\"cid:123\"></p>"),
//!     ))?;
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
//! use lettre::message::{header, Attachment, Body, Message, MultiPart, SinglePart};
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
//!                     .singlepart(SinglePart::plain(String::from("Hello, world! :)")))
//!                     .multipart(
//!                         MultiPart::related()
//!                             .singlepart(SinglePart::html(String::from(
//!                                 "<p><b>Hello</b>, <i>world</i>! <img src=cid:123></p>",
//!                             )))
//!                             .singlepart(
//!                                 Attachment::new_inline(String::from("123"))
//!                                     .body(image_body, "image/png".parse().unwrap()),
//!                             ),
//!                     ),
//!             )
//!             .singlepart(Attachment::new(String::from("example.rs")).body(
//!                 String::from("fn main() { println!(\"Hello, World!\") }"),
//!                 "text/plain".parse().unwrap(),
//!             )),
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

use std::{io::Write, iter};

pub use self::builder::{
    MessageBuilder, WantsBody, WantsEnvelopeOrSubject, WantsFrom, WantsRecipients1,
    WantsRecipients2, WantsRecipients3, WantsReplyTo, WantsSubject,
};
pub use attachment::Attachment;
pub use body::{Body, IntoBody, MaybeString};
pub use mailbox::*;
pub use mimebody::*;

mod attachment;
mod body;
mod builder;
pub mod header;
mod mailbox;
mod mimebody;

use crate::{address::Envelope, message::header::Headers};

const DEFAULT_MESSAGE_ID_DOMAIN: &str = "localhost";

/// Something that can be formatted as an email message
trait EmailFormat {
    // Use a writer?
    fn format(&self, out: &mut Vec<u8>);
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
    pub fn builder() -> MessageBuilder<WantsFrom> {
        MessageBuilder {
            state: WantsFrom(()),
        }
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
                out.extend_from_slice(r)
            }
        }
    }
}

/// Create a random message id.
/// (Not cryptographically random)
fn make_message_id() -> String {
    iter::repeat_with(fastrand::alphanumeric).take(36).collect()
}

#[cfg(test)]
mod test {
    use std::time::{Duration, SystemTime};

    use super::{header, mailbox::Mailbox, make_message_id, Message, MultiPart, SinglePart};

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
        // Tue, 15 Nov 1994 08:12:31 GMT
        let date = SystemTime::UNIX_EPOCH + Duration::from_secs(784887151);

        let email = Message::builder()
            .date(date)
            .bcc("hidden@example.com".parse().unwrap())
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
                "Date: Tue, 15 Nov 1994 08:12:31 -0000\r\n",
                "From: =?utf-8?b?0JrQsNC4?= <kayo@example.com>\r\n",
                "To: \"Pony O.P.\" <pony@domain.tld>\r\n",
                "Subject: =?utf-8?b?0Y/So9CwINC10Lsg0LHQtdC705nQvSE=?=\r\n",
                "Content-Transfer-Encoding: 7bit\r\n",
                "\r\n",
                "Happy new year!"
            )
        );
    }

    #[test]
    fn email_with_png() {
        // Tue, 15 Nov 1994 08:12:31 GMT
        let date = SystemTime::UNIX_EPOCH + Duration::from_secs(784887151);
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
                            .header(header::ContentType::TEXT_HTML)
                            .body(String::from(
                                "<p><b>Hello</b>, <i>world</i>! <img src=cid:123></p>",
                            )),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::parse("image/png").unwrap())
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
            if i == 7 || i == 9 || i == 14 || i == 233 {
                continue;
            }

            assert_eq!(line.0, line.1)
        }
    }

    #[test]
    fn test_make_message_id() {
        let mut ids = std::collections::HashSet::with_capacity(10);
        for _ in 0..1000 {
            ids.insert(make_message_id());
        }

        // Ensure there are no duplicates
        assert_eq!(1000, ids.len());

        // Ensure correct length
        for id in ids {
            assert_eq!(36, id.len());
        }
    }
}
