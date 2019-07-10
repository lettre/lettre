//! Lettre is a mailer written in Rust. lettre_email provides a simple email builder.
//!

#![doc(html_root_url = "https://docs.rs/lettre_email/0.9.2")]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

pub extern crate mime;

pub mod error;

use crate::error::Error;
pub use email::{Address, Header, Mailbox, MimeMessage, MimeMultipartType};
use lettre::{error::Error as LettreError, EmailAddress, Envelope, SendableEmail};
use mime::Mime;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use time::{now, Tm};
use uuid::Uuid;

/// Builds a `MimeMessage` structure
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PartBuilder {
    /// Message
    message: MimeMessage,
}

impl Default for PartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a message id
pub type MessageId = String;

/// Builds an `Email` structure
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct EmailBuilder {
    /// Message
    message: PartBuilder,
    /// The recipients' addresses for the mail header
    to: Vec<Address>,
    /// The sender addresses for the mail header
    from: Vec<Address>,
    /// The Cc addresses for the mail header
    cc: Vec<Address>,
    /// The Bcc addresses for the mail header
    bcc: Vec<Address>,
    /// The Reply-To addresses for the mail header
    reply_to: Vec<Address>,
    /// The In-Reply-To ids for the mail header
    in_reply_to: Vec<MessageId>,
    /// The References ids for the mail header
    references: Vec<MessageId>,
    /// The sender address for the mail header
    sender: Option<Mailbox>,
    /// The envelope
    envelope: Option<Envelope>,
    /// Date issued
    date_issued: bool,
    /// Message-ID
    message_id: Option<String>,
}

/// Simple email representation
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Email {
    /// Message
    message: Vec<u8>,
    /// Envelope
    envelope: Envelope,
    /// Message-ID
    message_id: String,
}

impl Into<SendableEmail> for Email {
    fn into(self) -> SendableEmail {
        SendableEmail::new(
            self.envelope.clone(),
            self.message_id.to_string(),
            self.message,
        )
    }
}

impl Email {
    /// Creates a new email builder
    pub fn builder() -> EmailBuilder {
        EmailBuilder::new()
    }
}

impl PartBuilder {
    /// Creates a new empty part
    pub fn new() -> PartBuilder {
        PartBuilder {
            message: MimeMessage::new_blank_message(),
        }
    }

    /// Adds a generic header
    pub fn header<A: Into<Header>>(mut self, header: A) -> PartBuilder {
        self.message.headers.insert(header.into());
        self
    }

    /// Sets the body
    pub fn body<S: Into<String>>(mut self, body: S) -> PartBuilder {
        self.message.body = body.into();
        self
    }

    /// Defines a `MimeMultipartType` value
    pub fn message_type(mut self, mime_type: MimeMultipartType) -> PartBuilder {
        self.message.message_type = Some(mime_type);
        self
    }

    /// Adds a `ContentType` header with the given MIME type
    pub fn content_type(self, content_type: &Mime) -> PartBuilder {
        self.header(("Content-Type", content_type.to_string()))
    }

    /// Adds a child part
    pub fn child(mut self, child: MimeMessage) -> PartBuilder {
        self.message.children.push(child);
        self
    }

    /// Gets built `MimeMessage`
    pub fn build(mut self) -> MimeMessage {
        self.message.update_headers();
        self.message
    }
}

impl EmailBuilder {
    /// Creates a new empty email
    pub fn new() -> EmailBuilder {
        EmailBuilder {
            message: PartBuilder::new(),
            to: vec![],
            from: vec![],
            cc: vec![],
            bcc: vec![],
            reply_to: vec![],
            in_reply_to: vec![],
            references: vec![],
            sender: None,
            envelope: None,
            date_issued: false,
            message_id: None,
        }
    }

    /// Sets the email body
    pub fn body<S: Into<String>>(mut self, body: S) -> EmailBuilder {
        self.message = self.message.body(body);
        self
    }

    /// Add a generic header
    pub fn header<A: Into<Header>>(mut self, header: A) -> EmailBuilder {
        self.message = self.message.header(header);
        self
    }

    /// Adds a `From` header and stores the sender address
    pub fn from<A: Into<Mailbox>>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into();
        self.from.push(Address::Mailbox(mailbox));
        self
    }

    /// Adds a `To` header and stores the recipient address
    pub fn to<A: Into<Mailbox>>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into();
        self.to.push(Address::Mailbox(mailbox));
        self
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn cc<A: Into<Mailbox>>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into();
        self.cc.push(Address::Mailbox(mailbox));
        self
    }

    /// Adds a `Bcc` header and stores the recipient address
    pub fn bcc<A: Into<Mailbox>>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into();
        self.bcc.push(Address::Mailbox(mailbox));
        self
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: Into<Mailbox>>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into();
        self.reply_to.push(Address::Mailbox(mailbox));
        self
    }

    /// Adds a `In-Reply-To` header
    pub fn in_reply_to(mut self, message_id: MessageId) -> EmailBuilder {
        self.in_reply_to.push(message_id);
        self
    }

    /// Adds a `References` header
    pub fn references(mut self, message_id: MessageId) -> EmailBuilder {
        self.references.push(message_id);
        self
    }

    /// Adds a `Sender` header
    pub fn sender<A: Into<Mailbox>>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into();
        self.sender = Some(mailbox);
        self
    }

    /// Adds a `Subject` header
    pub fn subject<S: AsRef<str>>(mut self, subject: S) -> EmailBuilder {
        let subject = format!("=?utf-8?B?{}?=", base64::encode(subject.as_ref()));
        self.message = self.message.header(("Subject".to_string(), subject));
        self
    }

    /// Adds a `Date` header with the given date
    pub fn date(mut self, date: &Tm) -> EmailBuilder {
        self.message = self.message.header(("Date", Tm::rfc822z(date).to_string()));
        self.date_issued = true;
        self
    }

    /// Adds an attachment to the email from a file
    ///
    /// If not specified, the filename will be extracted from the file path.
    pub fn attachment_from_file(
        self,
        path: &Path,
        filename: Option<&str>,
        content_type: &Mime,
    ) -> Result<EmailBuilder, Error> {
        self.attachment(
            fs::read(path)?.as_slice(),
            filename.unwrap_or(
                path.file_name()
                    .and_then(OsStr::to_str)
                    .ok_or(Error::CannotParseFilename)?,
            ),
            content_type,
        )
    }

    /// Adds an attachment to the email from a vector of bytes.
    pub fn attachment(
        self,
        body: &[u8],
        filename: &str,
        content_type: &Mime,
    ) -> Result<EmailBuilder, Error> {
        let encoded_body = base64::encode(&body);
        let content = PartBuilder::new()
            .body(encoded_body)
            .header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename),
            ))
            .header(("Content-Type", content_type.to_string()))
            .header(("Content-Transfer-Encoding", "base64"))
            .build();

        Ok(self.message_type(MimeMultipartType::Mixed).child(content))
    }

    /// Set the message type
    pub fn message_type(mut self, message_type: MimeMultipartType) -> EmailBuilder {
        self.message = self.message.message_type(message_type);
        self
    }

    /// Adds a child
    pub fn child(mut self, child: MimeMessage) -> EmailBuilder {
        self.message = self.message.child(child);
        self
    }

    /// Sets the email body to plain text content
    pub fn text<S: Into<String>>(self, body: S) -> EmailBuilder {
        let text = PartBuilder::new()
            .body(body)
            .header(("Content-Type", mime::TEXT_PLAIN_UTF_8.to_string()))
            .build();
        self.child(text)
    }

    /// Sets the email body to HTML content
    pub fn html<S: Into<String>>(self, body: S) -> EmailBuilder {
        let html = PartBuilder::new()
            .body(body)
            .header(("Content-Type", mime::TEXT_HTML_UTF_8.to_string()))
            .build();
        self.child(html)
    }

    /// Sets the email content
    pub fn alternative<S: Into<String>, T: Into<String>>(
        self,
        body_html: S,
        body_text: T,
    ) -> EmailBuilder {
        let text = PartBuilder::new()
            .body(body_text)
            .header(("Content-Type", mime::TEXT_PLAIN_UTF_8.to_string()))
            .build();

        let html = PartBuilder::new()
            .body(body_html)
            .header(("Content-Type", mime::TEXT_HTML_UTF_8.to_string()))
            .build();

        let alternate = PartBuilder::new()
            .message_type(MimeMultipartType::Alternative)
            .child(text)
            .child(html);

        self.message_type(MimeMultipartType::Mixed)
            .child(alternate.build())
    }

    /// Sets the `Message-ID` header
    pub fn message_id<S: Clone + Into<String>>(mut self, id: S) -> EmailBuilder {
        self.message = self.message.header(("Message-ID", id.clone()));
        self.message_id = Some(id.into());
        self
    }

    /// Sets the envelope for manual destination control
    /// If this function is not called, the envelope will be calculated
    /// from the "to" and "cc" addresses you set.
    pub fn envelope(mut self, envelope: Envelope) -> EmailBuilder {
        self.envelope = Some(envelope);
        self
    }

    /// Builds the Email
    pub fn build(mut self) -> Result<Email, Error> {
        // If there are multiple addresses in "From", the "Sender" is required.
        if self.from.len() >= 2 && self.sender.is_none() {
            // So, we must find something to put as Sender.
            for possible_sender in &self.from {
                // Only a mailbox can be used as sender, not Address::Group.
                if let Address::Mailbox(ref mbx) = *possible_sender {
                    self.sender = Some(mbx.clone());
                    break;
                }
            }
            // Address::Group is not yet supported, so the line below will never panic.
            // If groups are supported one day, add another Error for this case
            //  and return it here, if sender_header is still None at this point.
            assert!(self.sender.is_some());
        }
        // Add the sender header, if any.
        if let Some(ref v) = self.sender {
            self.message = self.message.header(("Sender", v.to_string()));
        }
        // Calculate the envelope
        let envelope = match self.envelope {
            Some(e) => e,
            None => {
                // we need to generate the envelope
                let mut to = vec![];
                // add all receivers in to_header and cc_header
                for receiver in self.to.iter().chain(self.cc.iter()).chain(self.bcc.iter()) {
                    match *receiver {
                        Address::Mailbox(ref m) => to.push(EmailAddress::from_str(&m.address)?),
                        Address::Group(_, ref ms) => {
                            for m in ms.iter() {
                                to.push(EmailAddress::from_str(&m.address.clone())?);
                            }
                        }
                    }
                }
                let from = Some(EmailAddress::from_str(&match self.sender {
                    Some(x) => Ok(x.address.clone()), // if we have a sender_header, use it
                    None => {
                        // use a from header
                        debug_assert!(self.from.len() <= 1); // else we'd have sender_header
                        match self.from.first() {
                            Some(a) => match *a {
                                // if we have a from header
                                Address::Mailbox(ref mailbox) => Ok(mailbox.address.clone()), // use it
                                Address::Group(_, ref mailbox_list) => match mailbox_list.first() {
                                    // if it's an author group, use the first author
                                    Some(mailbox) => Ok(mailbox.address.clone()),
                                    // for an empty author group (the rarest of the rare cases)
                                    None => Err(Error::Envelope(LettreError::MissingFrom)), // empty envelope sender
                                },
                            },
                            // if we don't have a from header
                            None => Err(Error::Envelope(LettreError::MissingFrom)), // empty envelope sender
                        }
                    }
                }?)?);
                Envelope::new(from, to)?
            }
        };
        // Add the collected addresses as mailbox-list all at once.
        // The unwraps are fine because the conversions for Vec<Address> never errs.
        if !self.to.is_empty() {
            self.message = self
                .message
                .header(Header::new_with_value("To".into(), self.to).unwrap());
        }
        if !self.from.is_empty() {
            self.message = self
                .message
                .header(Header::new_with_value("From".into(), self.from).unwrap());
        } else if let Some(from) = envelope.from() {
            let from = vec![Address::new_mailbox(from.to_string())];
            self.message = self
                .message
                .header(Header::new_with_value("From".into(), from).unwrap());
        } else {
            Err(Error::Envelope(LettreError::MissingFrom))?;
        }
        if !self.cc.is_empty() {
            self.message = self
                .message
                .header(Header::new_with_value("Cc".into(), self.cc).unwrap());
        }
        if !self.reply_to.is_empty() {
            self.message = self
                .message
                .header(Header::new_with_value("Reply-To".into(), self.reply_to).unwrap());
        }
        if !self.in_reply_to.is_empty() {
            self.message = self.message.header(
                Header::new_with_value("In-Reply-To".into(), self.in_reply_to.join(" ")).unwrap(),
            );
        }
        if !self.references.is_empty() {
            self.message = self.message.header(
                Header::new_with_value("References".into(), self.references.join(" ")).unwrap(),
            );
        }

        if !self.date_issued {
            self.message = self
                .message
                .header(("Date", Tm::rfc822z(&now()).to_string()));
        }

        self.message = self.message.header(("MIME-Version", "1.0"));

        let message_id = match self.message_id {
            Some(id) => id,
            None => {
                let message_id = Uuid::new_v4();
                self.message = self
                    .message
                    .header(("Message-ID", format!("<{}.lettre@localhost>", message_id)));
                message_id.to_string()
            }
        };

        Ok(Email {
            message: self.message.build().as_string().into_bytes(),
            envelope,
            message_id,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{EmailBuilder, SendableEmail};
    use lettre::EmailAddress;
    use time::now;

    #[test]
    fn test_multiple_from() {
        let email_builder = EmailBuilder::new();
        let date_now = now();
        let email: SendableEmail = email_builder
            .to("anna@example.com")
            .from("dieter@example.com")
            .from("joachim@example.com")
            .date(&date_now)
            .subject("Invitation")
            .body("We invite you!")
            .build()
            .unwrap()
            .into();
        let id = email.message_id().to_string();
        assert_eq!(
            email.message_to_string().unwrap(),
            format!(
                "Date: {}\r\nSubject: =?utf-8?B?SW52aXRhdGlvbg==?=\r\nSender: \
                 <dieter@example.com>\r\nTo: <anna@example.com>\r\nFrom: \
                 <dieter@example.com>, <joachim@example.com>\r\nMIME-Version: \
                 1.0\r\nMessage-ID: <{}.lettre@localhost>\r\n\r\nWe invite you!\r\n",
                date_now.rfc822z(),
                id
            )
        );
    }

    #[test]
    fn test_email_builder() {
        let email_builder = EmailBuilder::new();
        let date_now = now();

        let email: SendableEmail = email_builder
            .to("user@localhost")
            .from("user@localhost")
            .cc(("cc@localhost", "Alias"))
            .bcc("bcc@localhost")
            .reply_to("reply@localhost")
            .in_reply_to("original".to_string())
            .sender("sender@localhost")
            .body("Hello World!")
            .date(&date_now)
            .subject("Hello")
            .header(("X-test", "value"))
            .build()
            .unwrap()
            .into();
        let id = email.message_id().to_string();
        assert_eq!(
            email.message_to_string().unwrap(),
            format!(
                "Date: {}\r\nSubject: =?utf-8?B?SGVsbG8=?=\r\nX-test: value\r\nSender: \
                 <sender@localhost>\r\nTo: <user@localhost>\r\nFrom: \
                 <user@localhost>\r\nCc: \"Alias\" <cc@localhost>\r\n\
                 Reply-To: <reply@localhost>\r\nIn-Reply-To: original\r\n\
                 MIME-Version: 1.0\r\nMessage-ID: \
                 <{}.lettre@localhost>\r\n\r\nHello World!\r\n",
                date_now.rfc822z(),
                id
            )
        );
    }

    #[test]
    fn test_custom_message_id() {
        let email_builder = EmailBuilder::new();
        let date_now = now();

        let email: SendableEmail = email_builder
            .to("user@localhost")
            .from("user@localhost")
            .cc(("cc@localhost", "Alias"))
            .bcc("bcc@localhost")
            .reply_to("reply@localhost")
            .in_reply_to("original".to_string())
            .sender("sender@localhost")
            .body("Hello World!")
            .date(&date_now)
            .subject("Hello")
            .header(("X-test", "value"))
            .message_id("my-shiny-id")
            .build()
            .unwrap()
            .into();
        assert_eq!(
            email.message_to_string().unwrap(),
            format!(
                "Date: {}\r\nSubject: Hello\r\nX-test: value\r\nMessage-ID: \
                 my-shiny-id\r\nSender: <sender@localhost>\r\nTo: <user@localhost>\r\nFrom: \
                 <user@localhost>\r\nCc: \"Alias\" <cc@localhost>\r\nReply-To: \
                 <reply@localhost>\r\nIn-Reply-To: original\r\nMIME-Version: 1.0\r\n\r\nHello \
                 World!\r\n",
                date_now.rfc822z()
            )
        );
    }

    #[test]
    fn test_email_sendable() {
        let email_builder = EmailBuilder::new();
        let date_now = now();

        let email: SendableEmail = email_builder
            .to("user@localhost")
            .from("user@localhost")
            .cc(("cc@localhost", "Alias"))
            .bcc("bcc@localhost")
            .reply_to("reply@localhost")
            .sender("sender@localhost")
            .body("Hello World!")
            .date(&date_now)
            .subject("Hello")
            .header(("X-test", "value"))
            .build()
            .unwrap()
            .into();

        assert_eq!(
            email.envelope().from().unwrap().to_string(),
            "sender@localhost".to_string()
        );
        assert_eq!(
            email.envelope().to(),
            vec![
                EmailAddress::new("user@localhost".to_string()).unwrap(),
                EmailAddress::new("cc@localhost".to_string()).unwrap(),
                EmailAddress::new("bcc@localhost".to_string()).unwrap(),
            ]
            .as_slice()
        );
    }

}
