//! Lettre is a mailer written in Rust. lettre_email provides a simple email builder.
//!

#![doc(html_root_url = "https://docs.rs/lettre_email/0.9.0")]
#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
        trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
        unused_qualifications)]

extern crate base64;
extern crate email as email_format;
extern crate lettre;
extern crate mime;
extern crate time;
extern crate uuid;

pub mod error;

pub use email_format::{Address, Header, Mailbox, MimeMessage, MimeMultipartType};
use error::Error;
use lettre::{EmailAddress, Envelope, Error as EmailError, SendableEmail};
use mime::Mime;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use time::{now, Tm};
use uuid::Uuid;
use std::str::FromStr;

/// Converts an address or an address with an alias to a `Header`
pub trait IntoHeader {
    /// Converts to a `Header` struct
    fn into_header(self) -> Header;
}

impl IntoHeader for Header {
    fn into_header(self) -> Header {
        self
    }
}

impl<S: Into<String>, T: Into<String>> IntoHeader for (S, T) {
    fn into_header(self) -> Header {
        let (name, value) = self;
        Header::new(name.into(), value.into())
    }
}

/// Converts an address or an address with an alias to a `Mailbox`
pub trait IntoMailbox {
    /// Converts to a `Mailbox` struct
    fn into_mailbox(self) -> Mailbox;
}

impl IntoMailbox for Mailbox {
    fn into_mailbox(self) -> Mailbox {
        self
    }
}

impl<'a> IntoMailbox for &'a str {
    fn into_mailbox(self) -> Mailbox {
        Mailbox::new(self.into())
    }
}

impl IntoMailbox for String {
    fn into_mailbox(self) -> Mailbox {
        Mailbox::new(self)
    }
}

impl<S: Into<String>, T: Into<String>> IntoMailbox for (S, T) {
    fn into_mailbox(self) -> Mailbox {
        let (address, alias) = self;
        Mailbox::new_with_name(alias.into(), address.into())
    }
}

/// Can be transformed to a sendable email
pub trait IntoEmail {
    /// Builds an email
    fn into_email(self) -> Result<Email, Error>;
}

impl IntoEmail for SimpleEmail {
    fn into_email(self) -> Result<Email, Error> {
        let mut builder = EmailBuilder::new();

        if self.from.is_some() {
            builder = builder.from(self.from.unwrap());
        }

        for to_address in self.to {
            builder = builder.to(to_address.into_mailbox());
        }

        for cc_address in self.cc {
            builder = builder.cc(cc_address.into_mailbox());
        }

        if self.reply_to.is_some() {
            builder = builder.reply_to(self.reply_to.unwrap().into_mailbox());
        }

        if self.subject.is_some() {
            builder = builder.subject(self.subject.unwrap());
        }

        // No date for now

        builder = match (self.text, self.html) {
            (Some(text), Some(html)) => builder.alternative(html, text),
            (Some(text), None) => builder.text(text),
            (None, Some(html)) => builder.html(html),
            (None, None) => builder,
        };

        for header in self.headers {
            builder = builder.header(header.into_header());
        }

        builder.build()
    }
}

/// Simple representation of an email, useful for some transports
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct SimpleEmail {
    from: Option<Mailbox>,
    to: Vec<Mailbox>,
    cc: Vec<Mailbox>,
    bcc: Vec<Mailbox>,
    reply_to: Option<Mailbox>,
    subject: Option<String>,
    date: Option<Tm>,
    html: Option<String>,
    text: Option<String>,
    attachments: Vec<String>,
    headers: Vec<Header>,
}

impl SimpleEmail {
    /// Adds a generic header
    pub fn header<A: IntoHeader>(mut self, header: A) -> SimpleEmail {
        self.headers.push(header.into_header());
        self
    }

    /// Adds a `From` header and stores the sender address
    pub fn from<A: IntoMailbox>(mut self, address: A) -> SimpleEmail {
        self.from = Some(address.into_mailbox());
        self
    }

    /// Adds a `To` header and stores the recipient address
    pub fn to<A: IntoMailbox>(mut self, address: A) -> SimpleEmail {
        self.to.push(address.into_mailbox());
        self
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn cc<A: IntoMailbox>(mut self, address: A) -> SimpleEmail {
        self.cc.push(address.into_mailbox());
        self
    }

    /// Adds a `Bcc` header and stores the recipient address
    pub fn bcc<A: IntoMailbox>(mut self, address: A) -> SimpleEmail {
        self.bcc.push(address.into_mailbox());
        self
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: IntoMailbox>(mut self, address: A) -> SimpleEmail {
        self.reply_to = Some(address.into_mailbox());
        self
    }

    /// Adds a `Subject` header
    pub fn subject<S: Into<String>>(mut self, subject: S) -> SimpleEmail {
        self.subject = Some(subject.into());
        self
    }

    /// Adds a `Date` header with the given date
    pub fn date(mut self, date: Tm) -> SimpleEmail {
        self.date = Some(date);
        self
    }

    /// Adds an attachment to the message
    pub fn attachment<S: Into<String>>(mut self, path: S) -> SimpleEmail {
        self.attachments.push(path.into());
        self
    }

    /// Sets the email body to plain text content
    pub fn text<S: Into<String>>(mut self, body: S) -> SimpleEmail {
        self.text = Some(body.into());
        self
    }

    /// Sets the email body to HTML content
    pub fn html<S: Into<String>>(mut self, body: S) -> SimpleEmail {
        self.html = Some(body.into());
        self
    }
}

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

/// Builds an `Email` structure
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct EmailBuilder {
    /// Message
    message: PartBuilder,
    /// The recipients' addresses for the mail header
    to_header: Vec<Address>,
    /// The sender addresses for the mail header
    from_header: Vec<Address>,
    /// The Cc addresses for the mail header
    cc_header: Vec<Address>,
    /// The Bcc addresses for the mail header
    bcc_header: Vec<Address>,
    /// The Reply-To addresses for the mail header
    reply_to_header: Vec<Address>,
    /// The sender address for the mail header
    sender_header: Option<Mailbox>,
    /// The envelope
    envelope: Option<Envelope>,
    /// Date issued
    date_issued: bool,
}

/// Simple email representation
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Email {
    /// Message
    message: Vec<u8>,
    /// Envelope
    envelope: Envelope,
    /// Message-ID
    message_id: Uuid,
}

impl PartBuilder {
    /// Creates a new empty part
    pub fn new() -> PartBuilder {
        PartBuilder {
            message: MimeMessage::new_blank_message(),
        }
    }

    /// Adds a generic header
    pub fn header<A: IntoHeader>(mut self, header: A) -> PartBuilder {
        self.message.headers.insert(header.into_header());
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
        self.header(("Content-Type", format!("{}", content_type).as_ref()))
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
            to_header: vec![],
            from_header: vec![],
            cc_header: vec![],
            bcc_header: vec![],
            reply_to_header: vec![],
            sender_header: None,
            envelope: None,
            date_issued: false,
        }
    }

    /// Sets the email body
    pub fn body<S: Into<String>>(mut self, body: S) -> EmailBuilder {
        self.message = self.message.body(body);
        self
    }

    /// Add a generic header
    pub fn header<A: IntoHeader>(mut self, header: A) -> EmailBuilder {
        self.message = self.message.header(header);
        self
    }

    /// Adds a `From` header and stores the sender address
    pub fn from<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into_mailbox();
        self.from_header.push(Address::Mailbox(mailbox));
        self
    }

    /// Adds a `To` header and stores the recipient address
    pub fn to<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into_mailbox();
        self.to_header.push(Address::Mailbox(mailbox));
        self
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn cc<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into_mailbox();
        self.cc_header.push(Address::Mailbox(mailbox));
        self
    }

    /// Adds a `Bcc` header and stores the recipient address
    pub fn bcc<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into_mailbox();
        self.bcc_header.push(Address::Mailbox(mailbox));
        self
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into_mailbox();
        self.reply_to_header.push(Address::Mailbox(mailbox));
        self
    }

    /// Adds a `Sender` header
    pub fn sender<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.into_mailbox();
        self.sender_header = Some(mailbox);
        self
    }

    /// Adds a `Subject` header
    pub fn subject<S: Into<String>>(mut self, subject: S) -> EmailBuilder {
        self.message = self.message.header(("Subject".to_string(), subject.into()));
        self
    }

    /// Adds a `Date` header with the given date
    pub fn date(mut self, date: &Tm) -> EmailBuilder {
        self.message = self.message.header(("Date", Tm::rfc822z(date).to_string()));
        self.date_issued = true;
        self
    }

    /// Adds an attachment to the email
    pub fn attachment(
        self,
        path: &Path,
        filename: Option<&str>,
        content_type: &Mime,
    ) -> Result<EmailBuilder, Error> {
        let file = File::open(path);
        let body = match file {
            Ok(mut f) => {
                let mut data = Vec::new();
                let read = f.read_to_end(&mut data);
                match read {
                    Ok(_) => data,
                    Err(e) => {
                        return Err(From::from(e));
                    }
                }
            }
            Err(e) => {
                return Err(From::from(e));
            }
        };

        let actual_filename = match filename {
            Some(name) => name,
            None => match path.file_name() {
                Some(name) => match name.to_str() {
                    Some(name) => name,
                    None => return Err(Error::CannotParseFilename),
                },
                None => return Err(Error::CannotParseFilename),
            },
        };

        let encoded_body = base64::encode(&body);
        let content = PartBuilder::new()
            .body(encoded_body)
            .header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", actual_filename),
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
            .header((
                "Content-Type",
                format!("{}", mime::TEXT_PLAIN_UTF_8).as_ref(),
            ))
            .build();
        self.child(text)
    }


    /// Sets the email body to HTML content
    pub fn html<S: Into<String>>(self, body: S) -> EmailBuilder {
        let html = PartBuilder::new()
            .body(body)
            .header((
                "Content-Type",
                format!("{}", mime::TEXT_HTML_UTF_8).as_ref(),
            ))
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
            .header((
                "Content-Type",
                format!("{}", mime::TEXT_PLAIN_UTF_8).as_ref(),
            ))
            .build();

        let html = PartBuilder::new()
            .body(body_html)
            .header((
                "Content-Type",
                format!("{}", mime::TEXT_HTML_UTF_8).as_ref(),
            ))
            .build();

        let alternate = PartBuilder::new().message_type(MimeMultipartType::Alternative)
            .child(text).child(html);

        self.message_type(MimeMultipartType::Mixed).child(alternate.build())
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
        if self.from_header.len() >= 2 && self.sender_header.is_none() {
            // So, we must find something to put as Sender.
            for possible_sender in &self.from_header {
                // Only a mailbox can be used as sender, not Address::Group.
                if let Address::Mailbox(ref mbx) = *possible_sender {
                    self.sender_header = Some(mbx.clone());
                    break;
                }
            }
            // Address::Group is not yet supported, so the line below will never panic.
            // If groups are supported one day, add another Error for this case
            //  and return it here, if sender_header is still None at this point.
            assert!(self.sender_header.is_some());
        }
        // Add the sender header, if any.
        if let Some(ref v) = self.sender_header {
            self . message = self.message.header(("Sender", v.to_string().as_ref()));
        }
        // Calculate the envelope
        let envelope = match self.envelope {
            Some(e) => e,
            None => {
                // we need to generate the envelope
                let mut e = Envelope::builder();
                // add all receivers in to_header and cc_header
                for receiver in self.to_header
                    .iter()
                    .chain(self.cc_header.iter())
                    .chain(self.bcc_header.iter())
                {
                    match *receiver {
                        Address::Mailbox(ref m) => e.add_to(EmailAddress::from_str(&m.address)?),
                        Address::Group(_, ref ms) => for m in ms.iter() {
                            e.add_to(EmailAddress::from_str(&m.address.clone())?);
                        },
                    }
                }
                e.set_from(EmailAddress::from_str(&match self.sender_header {
                    Some(x) => x.address.clone(), // if we have a sender_header, use it
                    None => {
                        // use a from header
                        debug_assert!(self.from_header.len() <= 1); // else we'd have sender_header
                        match self.from_header.first() {
                            Some(a) => match *a {
                                // if we have a from header
                                Address::Mailbox(ref mailbox) => mailbox.address.clone(), // use it
                                Address::Group(_, ref mailbox_list) => match mailbox_list.first() {
                                    // if it's an author group, use the first author
                                    Some(mailbox) => mailbox.address.clone(),
                                    // for an empty author group (the rarest of the rare cases)
                                    None => return Err(Error::Email(EmailError::MissingFrom)), // empty envelope sender
                                },
                            },
                            // if we don't have a from header
                            None => return Err(Error::Email(EmailError::MissingFrom)), // empty envelope sender
                        }
                    }
                })?);
                e.build()?
            }
        };
        // Add the collected addresses as mailbox-list all at once.
        // The unwraps are fine because the conversions for Vec<Address> never errs.
        if !self.to_header.is_empty() {
            self.message = self.message
                .header(Header::new_with_value("To".into(), self.to_header).unwrap());
        }
        if !self.from_header.is_empty() {
            self.message = self.message
                .header(Header::new_with_value("From".into(), self.from_header).unwrap());
        } else {
            return Err(Error::Email(EmailError::MissingFrom));
        }
        if !self.cc_header.is_empty() {
            self.message = self.message
                .header(Header::new_with_value("Cc".into(), self.cc_header).unwrap());
        }
        if !self.bcc_header.is_empty() {
            self.message = self.message
                .header(Header::new_with_value("Bcc".into(), self.bcc_header).unwrap());
        }
        if !self.reply_to_header.is_empty() {
            self.message = self.message.header(
                Header::new_with_value("Reply-To".into(), self.reply_to_header).unwrap(),
            );
        }

        if !self.date_issued {
            self.message = self.message
                .header(("Date", Tm::rfc822z(&now()).to_string().as_ref()));
        }

        self.message = self.message.header(("MIME-Version", "1.0"));

        let message_id = Uuid::new_v4();

        if let Ok(header) = Header::new_with_value(
            "Message-ID".to_string(),
            format!("<{}.lettre@localhost>", message_id),
        ) {
            self.message = self.message.header(header)
        }

        Ok(Email {
            message: self.message.build().as_string().into_bytes(),
            envelope,
            message_id,
        })
    }
}

impl<'a> SendableEmail<'a, &'a [u8]> for Email {
    fn envelope(&self) -> Envelope {
        self.envelope.clone()
    }

    fn message_id(&self) -> String {
        self.message_id.to_string()
    }

    fn message(&'a self) -> Box<&[u8]> {
        Box::new(self.message.as_slice())
    }
}

#[cfg(test)]
mod test {
    use super::EmailBuilder;
    use lettre::{EmailAddress, SendableEmail};
    use time::now;

    #[test]
    fn test_multiple_from() {
        let email_builder = EmailBuilder::new();
        let date_now = now();
        let email = email_builder
            .to("anna@example.com")
            .from("dieter@example.com")
            .from("joachim@example.com")
            .date(&date_now)
            .subject("Invitation")
            .body("We invite you!")
            .build()
            .unwrap();
        assert_eq!(
            format!("{}", String::from_utf8_lossy(email.message().as_ref())),
            format!(
                "Date: {}\r\nSubject: Invitation\r\nSender: \
                 <dieter@example.com>\r\nTo: <anna@example.com>\r\nFrom: \
                 <dieter@example.com>, <joachim@example.com>\r\nMIME-Version: \
                 1.0\r\nMessage-ID: <{}.lettre@localhost>\r\n\r\nWe invite you!\r\n",
                date_now.rfc822z(),
                email.message_id()
            )
        );
    }

    #[test]
    fn test_email_builder() {
        let email_builder = EmailBuilder::new();
        let date_now = now();

        let email = email_builder
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
            .unwrap();

        assert_eq!(
            format!("{}", String::from_utf8_lossy(email.message().as_ref())),
            format!(
                "Date: {}\r\nSubject: Hello\r\nX-test: value\r\nSender: \
                 <sender@localhost>\r\nTo: <user@localhost>\r\nFrom: \
                 <user@localhost>\r\nCc: \"Alias\" <cc@localhost>\r\n\
                 Bcc: <bcc@localhost>\r\nReply-To: <reply@localhost>\r\n\
                 MIME-Version: 1.0\r\nMessage-ID: \
                 <{}.lettre@localhost>\r\n\r\nHello World!\r\n",
                date_now.rfc822z(),
                email.message_id()
            )
        );
    }

    #[test]
    fn test_email_sendable() {
        let email_builder = EmailBuilder::new();
        let date_now = now();

        let email = email_builder
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
            .unwrap();

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
            ].as_slice()
        );
    }

}
