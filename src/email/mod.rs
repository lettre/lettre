//! Simple email representation
pub mod error;

use email::error::Error;

use email_format::{Header, Mailbox, MimeMessage, MimeMultipartType};
use mime::Mime;
use std::fmt;
use std::fmt::{Display, Formatter};
use time::{Tm, now};
use uuid::Uuid;

/// Converts an address or an address with an alias to a `Header`
pub trait ToHeader {
    /// Converts to a `Header` struct
    fn to_header(&self) -> Header;
}

impl ToHeader for Header {
    fn to_header(&self) -> Header {
        self.clone()
    }
}

impl<'a> ToHeader for (&'a str, &'a str) {
    fn to_header(&self) -> Header {
        let (name, value) = *self;
        Header::new(name.to_string(), value.to_string())
    }
}

/// Converts an adress or an address with an alias to a `Mailbox`
pub trait ToMailbox {
    /// Converts to a `Mailbox` struct
    fn to_mailbox(&self) -> Mailbox;
}

impl ToMailbox for Mailbox {
    fn to_mailbox(&self) -> Mailbox {
        (*self).clone()
    }
}

impl<'a> ToMailbox for &'a str {
    fn to_mailbox(&self) -> Mailbox {
        Mailbox::new(self.to_string())
    }
}

impl<'a> ToMailbox for (&'a str, &'a str) {
    fn to_mailbox(&self) -> Mailbox {
        let (address, alias) = *self;
        Mailbox::new_with_name(alias.to_string(), address.to_string())
    }
}

/// Can be transformed to a sendable email
pub trait IntoEmail {
    /// Builds an email
    fn into_email(&self) -> Result<Email, Error>;
}

impl IntoEmail for SimpleEmail {
    fn into_email(&self) -> Result<Email, Error> {
        let mut builder = EmailBuilder::new();

        if self.from.is_some() {
            builder.add_from(self.from.as_ref().unwrap().as_str().to_mailbox());
        }

        for to_address in self.to.as_slice() {
            builder.add_to(to_address.as_str().to_mailbox());
        }

        for cc_address in self.cc.as_slice() {
            builder.add_cc(cc_address.as_str().to_mailbox());
        }

        // No bcc for now

        if self.reply_to.is_some() {
            builder.add_reply_to(self.reply_to.as_ref().unwrap().as_str().to_mailbox());
        }

        if self.subject.is_some() {
            builder.set_subject(self.subject.as_ref().unwrap().as_str());
        }

        // No date for now

        match (self.text.as_ref(), self.html.as_ref()) {
            (Some(text), Some(html)) => builder.set_alternative(html.as_str(), text.as_str()),
            (Some(text), None) => builder.set_text(text.as_str()),
            (None, Some(html)) => builder.set_html(html.as_str()),
            (None, None) => (),
        }

        for header in self.headers.as_slice() {
            builder.add_header(header.to_header());
        }

        Ok(builder.build().unwrap())
    }
}


/// Simple representation of an email, useful for some transports
#[derive(PartialEq,Eq,Clone,Debug,Default)]
pub struct SimpleEmail {
    from: Option<String>,
    to: Vec<String>,
    cc: Vec<String>,
    // bcc: Vec<String>,
    reply_to: Option<String>,
    subject: Option<String>,
    date: Option<Tm>,
    html: Option<String>,
    text: Option<String>,
    // attachments: Vec<String>,
    headers: Vec<Header>,
}

impl SimpleEmail {
    /// Adds a generic header
    pub fn header<A: ToHeader>(mut self, header: A) -> SimpleEmail {
        self.add_header(header);
        self
    }

    /// Adds a generic header
    pub fn add_header<A: ToHeader>(&mut self, header: A) {
        self.headers.push(header.to_header());
    }

    /// Adds a `From` header and stores the sender address
    pub fn from<A: ToMailbox>(mut self, address: A) -> SimpleEmail {
        self.add_from(address);
        self
    }

    /// Adds a `From` header and stores the sender address
    pub fn add_from<A: ToMailbox>(&mut self, address: A) {
        let mailbox = address.to_mailbox();
        self.from = Some(mailbox.address);
    }

    /// Adds a `To` header and stores the recipient address
    pub fn to<A: ToMailbox>(mut self, address: A) -> SimpleEmail {
        self.add_to(address);
        self
    }

    /// Adds a `To` header and stores the recipient address
    pub fn add_to<A: ToMailbox>(&mut self, address: A) {
        let mailbox = address.to_mailbox();
        self.to.push(mailbox.address);
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn cc<A: ToMailbox>(mut self, address: A) -> SimpleEmail {
        self.add_cc(address);
        self
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn add_cc<A: ToMailbox>(&mut self, address: A) {
        let mailbox = address.to_mailbox();
        self.cc.push(mailbox.address);
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: ToMailbox>(mut self, address: A) -> SimpleEmail {
        self.add_reply_to(address);
        self
    }

    /// Adds a `Reply-To` header
    pub fn add_reply_to<A: ToMailbox>(&mut self, address: A) {
        let mailbox = address.to_mailbox();
        self.reply_to = Some(mailbox.address);
    }

    /// Adds a `Subject` header
    pub fn subject(mut self, subject: &str) -> SimpleEmail {
        self.set_subject(subject);
        self
    }

    /// Adds a `Subject` header
    pub fn set_subject(&mut self, subject: &str) {
        self.subject = Some(subject.to_string());
    }

    /// Adds a `Date` header with the given date
    pub fn date(mut self, date: &Tm) -> SimpleEmail {
        self.set_date(date);
        self
    }

    /// Adds a `Date` header with the given date
    pub fn set_date(&mut self, date: &Tm) {
        self.date = Some(date.clone());
    }

    /// Sets the email body to plain text content
    pub fn text(mut self, body: &str) -> SimpleEmail {
        self.set_text(body);
        self
    }

    /// Sets the email body to plain text content
    pub fn set_text(&mut self, body: &str) {
        self.text = Some(body.to_string());
    }

    /// Sets the email body to HTML content
    pub fn html(mut self, body: &str) -> SimpleEmail {
        self.set_html(body);
        self
    }

    /// Sets the email body to HTML content
    pub fn set_html(&mut self, body: &str) {
        self.html = Some(body.to_string());
    }
}

/// Builds a `MimeMessage` structure
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct PartBuilder {
    /// Message
    message: MimeMessage,
}

/// Builds an `Email` structure
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct EmailBuilder {
    /// Message
    message: PartBuilder,
    /// The envelope recipients' addresses
    to: Vec<String>,
    /// The envelope sender address
    from: Option<String>,
    /// Date issued
    date_issued: bool,
}

/// Simple email enveloppe representation
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct Envelope {
    /// The envelope recipients' addresses
    to: Vec<String>,
    /// The envelope sender address
    from: String,
}

/// Simple email representation
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct Email {
    /// Message
    message: MimeMessage,
    /// Envelope
    envelope: Envelope,
    /// Message-ID
    message_id: Uuid,
}

impl Display for Email {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.message.as_string())
    }
}

impl PartBuilder {
    /// Creates a new empty part
    pub fn new() -> PartBuilder {
        PartBuilder { message: MimeMessage::new_blank_message() }
    }

    /// Adds a generic header
    pub fn header<A: ToHeader>(mut self, header: A) -> PartBuilder {
        self.add_header(header);
        self
    }

    /// Adds a generic header
    pub fn add_header<A: ToHeader>(&mut self, header: A) {
        self.message.headers.insert(header.to_header());
    }

    /// Sets the body
    pub fn body(mut self, body: &str) -> PartBuilder {
        self.set_body(body);
        self
    }

    /// Sets the body
    pub fn set_body(&mut self, body: &str) {
        self.message.body = body.to_string();
    }


    /// Defines a `MimeMultipartType` value
    pub fn message_type(mut self, mime_type: MimeMultipartType) -> PartBuilder {
        self.set_message_type(mime_type);
        self
    }

    /// Defines a `MimeMultipartType` value
    pub fn set_message_type(&mut self, mime_type: MimeMultipartType) {
        self.message.message_type = Some(mime_type);
    }

    /// Adds a `ContentType` header with the given MIME type
    pub fn content_type(mut self, content_type: Mime) -> PartBuilder {
        self.set_content_type(content_type);
        self
    }

    /// Adds a `ContentType` header with the given MIME type
    pub fn set_content_type(&mut self, content_type: Mime) {
        self.add_header(("Content-Type", format!("{}", content_type).as_ref()));
    }

    /// Adds a child part
    pub fn child(mut self, child: MimeMessage) -> PartBuilder {
        self.add_child(child);
        self
    }

    /// Adds a child part
    pub fn add_child(&mut self, child: MimeMessage) {
        self.message.children.push(child);
    }

    /// Gets builded `MimeMessage`
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
            from: None,
            date_issued: false,
        }
    }

    /// Sets the email body
    pub fn body(mut self, body: &str) -> EmailBuilder {
        self.message.set_body(body);
        self
    }

    /// Sets the email body
    pub fn set_body(&mut self, body: &str) {
        self.message.set_body(body);
    }

    /// Add a generic header
    pub fn header<A: ToHeader>(mut self, header: A) -> EmailBuilder {
        self.message.add_header(header);
        self
    }

    /// Add a generic header
    pub fn add_header<A: ToHeader>(&mut self, header: A) {
        self.message.add_header(header);
    }

    /// Adds a `From` header and stores the sender address
    pub fn from<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        self.add_from(address);
        self
    }

    /// Adds a `From` header and stores the sender address
    pub fn add_from<A: ToMailbox>(&mut self, address: A) {
        let mailbox = address.to_mailbox();
        self.message.add_header(("From", mailbox.to_string().as_ref()));
        self.from = Some(mailbox.address);
    }

    /// Adds a `To` header and stores the recipient address
    pub fn to<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        self.add_to(address);
        self
    }

    /// Adds a `To` header and stores the recipient address
    pub fn add_to<A: ToMailbox>(&mut self, address: A) {
        let mailbox = address.to_mailbox();
        self.message.add_header(("To", mailbox.to_string().as_ref()));
        self.to.push(mailbox.address);
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn cc<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        self.add_cc(address);
        self
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn add_cc<A: ToMailbox>(&mut self, address: A) {
        let mailbox = address.to_mailbox();
        self.message.add_header(("Cc", mailbox.to_string().as_ref()));
        self.to.push(mailbox.address);
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        self.add_reply_to(address);
        self
    }

    /// Adds a `Reply-To` header
    pub fn add_reply_to<A: ToMailbox>(&mut self, address: A) {
        let mailbox = address.to_mailbox();
        self.message.add_header(("Reply-To", mailbox.to_string().as_ref()));
    }

    /// Adds a `Sender` header
    pub fn sender<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        self.set_sender(address);
        self
    }

    /// Adds a `Sender` header
    pub fn set_sender<A: ToMailbox>(&mut self, address: A) {
        let mailbox = address.to_mailbox();
        self.message.add_header(("Sender", mailbox.to_string().as_ref()));
        self.from = Some(mailbox.address);
    }

    /// Adds a `Subject` header
    pub fn subject(mut self, subject: &str) -> EmailBuilder {
        self.set_subject(subject);
        self
    }

    /// Adds a `Subject` header
    pub fn set_subject(&mut self, subject: &str) {
        self.message.add_header(("Subject", subject));
    }

    /// Adds a `Date` header with the given date
    pub fn date(mut self, date: &Tm) -> EmailBuilder {
        self.set_date(date);
        self
    }

    /// Adds a `Date` header with the given date
    pub fn set_date(&mut self, date: &Tm) {
        self.message.add_header(("Date", Tm::rfc822z(date).to_string().as_ref()));
        self.date_issued = true;
    }

    /// Set the message type
    pub fn message_type(mut self, message_type: MimeMultipartType) -> EmailBuilder {
        self.set_message_type(message_type);
        self
    }

    /// Set the message type
    pub fn set_message_type(&mut self, message_type: MimeMultipartType) {
        self.message.set_message_type(message_type);
    }

    /// Adds a child
    pub fn child(mut self, child: MimeMessage) -> EmailBuilder {
        self.add_child(child);
        self
    }

    /// Adds a child
    pub fn add_child(&mut self, child: MimeMessage) {
        self.message.add_child(child);
    }

    /// Sets the email body to plain text content
    pub fn text(mut self, body: &str) -> EmailBuilder {
        self.set_text(body);
        self
    }

    /// Sets the email body to plain text content
    pub fn set_text(&mut self, body: &str) {
        self.message.set_body(body);
        self.message
            .add_header(("Content-Type", format!("{}", mime!(Text/Plain; Charset=Utf8)).as_ref()));
    }

    /// Sets the email body to HTML content
    pub fn html(mut self, body: &str) -> EmailBuilder {
        self.set_html(body);
        self
    }

    /// Sets the email body to HTML content
    pub fn set_html(&mut self, body: &str) {
        self.message.set_body(body);
        self.message
            .add_header(("Content-Type", format!("{}", mime!(Text/Html; Charset=Utf8)).as_ref()));
    }

    /// Sets the email content
    pub fn alternative(mut self, body_html: &str, body_text: &str) -> EmailBuilder {
        self.set_alternative(body_html, body_text);
        self
    }

    /// Sets the email content
    pub fn set_alternative(&mut self, body_html: &str, body_text: &str) {
        let mut alternate = PartBuilder::new();
        alternate.set_message_type(MimeMultipartType::Alternative);

        let text = PartBuilder::new()
            .body(body_text)
            .header(("Content-Type", format!("{}", mime!(Text/Plain; Charset=Utf8)).as_ref()))
            .build();

        let html = PartBuilder::new()
            .body(body_html)
            .header(("Content-Type", format!("{}", mime!(Text/Html; Charset=Utf8)).as_ref()))
            .build();

        alternate.add_child(text);
        alternate.add_child(html);

        self.set_message_type(MimeMultipartType::Mixed);
        self.add_child(alternate.build());
    }

    /// Builds the Email
    pub fn build(mut self) -> Result<Email, Error> {
        if self.from.is_none() {
            return Err(Error::MissingFrom);
        }
        if self.to.is_empty() {
            return Err(Error::MissingTo);
        }

        if !self.date_issued {
            self.message.add_header(("Date", Tm::rfc822z(&now()).to_string().as_ref()));
        }

        self.message.add_header(("MIME-Version", "1.0"));

        let message_id = Uuid::new_v4();

        if let Ok(header) = Header::new_with_value("Message-ID".to_string(),
                                                   format!("<{}.lettre@localhost>", message_id)) {
            self.message.add_header(header)
        }

        Ok(Email {
            message: self.message.build(),
            envelope: Envelope {
                to: self.to,
                from: self.from.unwrap(),
            },
            message_id: message_id,
        })
    }
}

/// Email sendable by an SMTP client
pub trait SendableEmail {
    /// From address
    fn from_address(&self) -> String;
    /// To addresses
    fn to_addresses(&self) -> Vec<String>;
    /// Message content
    fn message(&self) -> String;
    /// Message ID
    fn message_id(&self) -> String;
}

/// Minimal email structure
pub struct SimpleSendableEmail {
    /// From address
    from: String,
    /// To addresses
    to: Vec<String>,
    /// Message
    message: String,
}

impl SimpleSendableEmail {
    /// Returns a new email
    pub fn new(from_address: &str, to_address: Vec<String>, message: &str) -> SimpleSendableEmail {
        SimpleSendableEmail {
            from: from_address.to_string(),
            to: to_address,
            message: message.to_string(),
        }
    }
}

impl SendableEmail for SimpleSendableEmail {
    fn from_address(&self) -> String {
        self.from.clone()
    }

    fn to_addresses(&self) -> Vec<String> {
        self.to.clone()
    }

    fn message(&self) -> String {
        self.message.clone()
    }

    fn message_id(&self) -> String {
        format!("{}", Uuid::new_v4())
    }
}

impl SendableEmail for Email {
    fn to_addresses(&self) -> Vec<String> {
        self.envelope.to.clone()
    }

    fn from_address(&self) -> String {
        self.envelope.from.clone()
    }

    fn message(&self) -> String {
        format!("{}", self)
    }

    fn message_id(&self) -> String {
        format!("{}", self.message_id)
    }
}

#[cfg(test)]
mod test {
    use email_format::{Header, MimeMessage};

    use super::{Email, EmailBuilder, Envelope, SendableEmail};
    use time::now;

    use uuid::Uuid;

    #[test]
    fn test_email_display() {
        let current_message = Uuid::new_v4();

        let mut email = Email {
            message: MimeMessage::new_blank_message(),
            envelope: Envelope {
                to: vec![],
                from: "".to_string(),
            },
            message_id: current_message,
        };

        email.message.headers.insert(Header::new_with_value("Message-ID".to_string(),
                                                            format!("<{}@rust-smtp>",
                                                                    current_message))
            .unwrap());

        email.message
            .headers
            .insert(Header::new_with_value("To".to_string(), "to@example.com".to_string())
                .unwrap());

        email.message.body = "body".to_string();

        assert_eq!(format!("{}", email),
                   format!("Message-ID: <{}@rust-smtp>\r\nTo: to@example.com\r\n\r\nbody\r\n",
                           current_message));
        assert_eq!(current_message.to_string(), email.message_id());
    }

    #[test]
    fn test_simple_email_builder() {
        let email_builder = EmailBuilder::new();
        let date_now = now();

        let email = email_builder.to("user@localhost")
            .from("user@localhost")
            .cc(("cc@localhost", "Alias"))
            .reply_to("reply@localhost")
            .sender("sender@localhost")
            .body("Hello World!")
            .date(&date_now)
            .subject("Hello")
            .header(("X-test", "value"))
            .build()
            .unwrap();

        assert_eq!(format!("{}", email),
                   format!("To: <user@localhost>\r\nFrom: <user@localhost>\r\nCc: \"Alias\" \
                            <cc@localhost>\r\nReply-To: <reply@localhost>\r\nSender: \
                            <sender@localhost>\r\nDate: {}\r\nSubject: Hello\r\nX-test: \
                            value\r\nMIME-Version: 1.0\r\nMessage-ID: \
                            <{}.lettre@localhost>\r\n\r\nHello World!\r\n",
                           date_now.rfc822z(),
                           email.message_id()));
    }

    #[test]
    fn test_email_sendable() {
        let email_builder = EmailBuilder::new();
        let date_now = now();

        let email = email_builder.to("user@localhost")
            .from("user@localhost")
            .cc(("cc@localhost", "Alias"))
            .reply_to("reply@localhost")
            .sender("sender@localhost")
            .body("Hello World!")
            .date(&date_now)
            .subject("Hello")
            .header(("X-test", "value"))
            .build()
            .unwrap();

        assert_eq!(email.from_address(), "sender@localhost".to_string());
        assert_eq!(email.to_addresses(),
                   vec!["user@localhost".to_string(), "cc@localhost".to_string()]);
        assert_eq!(email.message(), format!("{}", email));
    }

}
