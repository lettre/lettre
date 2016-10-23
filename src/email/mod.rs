//! Simple email representation

pub mod error;

use email::error::Error;
use email_format::{Address, Header, Mailbox, MimeMessage, MimeMultipartType};
use mime::Mime;
use std::fmt;
use std::fmt::{Display, Formatter};
use time::{Tm, now};
use uuid::Uuid;

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

/// Converts an adress or an address with an alias to a `Mailbox`
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
            builder.add_from(self.from.unwrap().into_mailbox());
        }

        for to_address in self.to {
            builder.add_to(to_address.into_mailbox());
        }

        for cc_address in self.cc {
            builder.add_cc(cc_address.into_mailbox());
        }

        // No bcc for now

        if self.reply_to.is_some() {
            builder.add_reply_to(self.reply_to.unwrap().into_mailbox());
        }

        if self.subject.is_some() {
            builder.set_subject(self.subject.unwrap());
        }

        // No date for now

        match (self.text, self.html) {
            (Some(text), Some(html)) => builder.set_alternative(html, text),
            (Some(text), None) => builder.set_text(text),
            (None, Some(html)) => builder.set_html(html),
            (None, None) => (),
        }

        for header in self.headers {
            builder.add_header(header.into_header());
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
    pub fn header<A: IntoHeader>(mut self, header: A) -> SimpleEmail {
        self.add_header(header);
        self
    }

    /// Adds a generic header
    pub fn add_header<A: IntoHeader>(&mut self, header: A) {
        self.headers.push(header.into_header());
    }

    /// Adds a `From` header and stores the sender address
    pub fn from<A: IntoMailbox>(mut self, address: A) -> SimpleEmail {
        self.add_from(address);
        self
    }

    /// Adds a `From` header and stores the sender address
    pub fn add_from<A: IntoMailbox>(&mut self, address: A) {
        let mailbox = address.into_mailbox();
        self.from = Some(mailbox.address);
    }

    /// Adds a `To` header and stores the recipient address
    pub fn to<A: IntoMailbox>(mut self, address: A) -> SimpleEmail {
        self.add_to(address);
        self
    }

    /// Adds a `To` header and stores the recipient address
    pub fn add_to<A: IntoMailbox>(&mut self, address: A) {
        let mailbox = address.into_mailbox();
        self.to.push(mailbox.address);
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn cc<A: IntoMailbox>(mut self, address: A) -> SimpleEmail {
        self.add_cc(address);
        self
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn add_cc<A: IntoMailbox>(&mut self, address: A) {
        let mailbox = address.into_mailbox();
        self.cc.push(mailbox.address);
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: IntoMailbox>(mut self, address: A) -> SimpleEmail {
        self.add_reply_to(address);
        self
    }

    /// Adds a `Reply-To` header
    pub fn add_reply_to<A: IntoMailbox>(&mut self, address: A) {
        let mailbox = address.into_mailbox();
        self.reply_to = Some(mailbox.address);
    }

    /// Adds a `Subject` header
    pub fn subject<S: Into<String>>(mut self, subject: S) -> SimpleEmail {
        self.set_subject(subject);
        self
    }

    /// Adds a `Subject` header
    pub fn set_subject<S: Into<String>>(&mut self, subject: S) {
        self.subject = Some(subject.into());
    }

    /// Adds a `Date` header with the given date
    pub fn date(mut self, date: Tm) -> SimpleEmail {
        self.set_date(date);
        self
    }

    /// Adds a `Date` header with the given date
    pub fn set_date(&mut self, date: Tm) {
        self.date = Some(date);
    }

    /// Sets the email body to plain text content
    pub fn text<S: Into<String>>(mut self, body: S) -> SimpleEmail {
        self.set_text(body);
        self
    }

    /// Sets the email body to plain text content
    pub fn set_text<S: Into<String>>(&mut self, body: S) {
        self.text = Some(body.into());
    }

    /// Sets the email body to HTML content
    pub fn html<S: Into<String>>(mut self, body: S) -> SimpleEmail {
        self.set_html(body);
        self
    }

    /// Sets the email body to HTML content
    pub fn set_html<S: Into<String>>(&mut self, body: S) {
        self.html = Some(body.into());
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
    /// The recipients' addresses for the mail header
    to_header: Vec<Address>,
    /// The sender addresses for the mail header
    from_header: Vec<Address>,
    /// The Cc addresses for the mail header
    cc_header: Vec<Address>,
    /// The Reply-To addresses for the mail header
    reply_to_header: Vec<Address>,
    /// The sender address for the mail header
    sender_header: Option<Mailbox>,
    /// The envelope
    envelope: Option<Envelope>,
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

impl Envelope {
    /// Constructs an envelope with no receivers and an empty sender
    pub fn new() -> Self {
        Envelope {
            to: vec![],
            from: String::new(),
        }
    }
    /// Adds a receiver
    pub fn to<S: Into<String>>(mut self, address: S) -> Self {
        self.add_to(address);
        self
    }
    /// Adds a receiver
    pub fn add_to<S: Into<String>>(&mut self, address: S) {
        self.to.push(address.into());
    }
    /// Sets the sender
    pub fn from<S: Into<String>>(mut self, address: S) -> Self {
        self.set_from(address);
        self
    }
    /// Sets the sender
    pub fn set_from<S: Into<String>>(&mut self, address: S) {
        self.from = address.into();
    }
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
    pub fn header<A: IntoHeader>(mut self, header: A) -> PartBuilder {
        self.add_header(header);
        self
    }

    /// Adds a generic header
    pub fn add_header<A: IntoHeader>(&mut self, header: A) {
        self.message.headers.insert(header.into_header());
    }

    /// Sets the body
    pub fn body<S: Into<String>>(mut self, body: S) -> PartBuilder {
        self.set_body(body);
        self
    }

    /// Sets the body
    pub fn set_body<S: Into<String>>(&mut self, body: S) {
        self.message.body = body.into();
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
            to_header: vec![],
            from_header: vec![],
            cc_header: vec![],
            reply_to_header: vec![],
            sender_header: None,
            envelope: None,
            date_issued: false,
        }
    }

    /// Sets the email body
    pub fn body<S: Into<String>>(mut self, body: S) -> EmailBuilder {
        self.message.set_body(body);
        self
    }

    /// Sets the email body
    pub fn set_body<S: Into<String>>(&mut self, body: S) {
        self.message.set_body(body);
    }

    /// Add a generic header
    pub fn header<A: IntoHeader>(mut self, header: A) -> EmailBuilder {
        self.message.add_header(header);
        self
    }

    /// Add a generic header
    pub fn add_header<A: IntoHeader>(&mut self, header: A) {
        self.message.add_header(header);
    }

    /// Adds a `From` header and stores the sender address
    pub fn from<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        self.add_from(address);
        self
    }

    /// Adds a `From` header and stores the sender address
    pub fn add_from<A: IntoMailbox>(&mut self, address: A) {
        let mailbox = address.into_mailbox();
        self.from_header.push(Address::Mailbox(mailbox));
    }

    /// Adds a `To` header and stores the recipient address
    pub fn to<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        self.add_to(address);
        self
    }

    /// Adds a `To` header and stores the recipient address
    pub fn add_to<A: IntoMailbox>(&mut self, address: A) {
        let mailbox = address.into_mailbox();
        self.to_header.push(Address::Mailbox(mailbox));
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn cc<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        self.add_cc(address);
        self
    }

    /// Adds a `Cc` header and stores the recipient address
    pub fn add_cc<A: IntoMailbox>(&mut self, address: A) {
        let mailbox = address.into_mailbox();
        self.cc_header.push(Address::Mailbox(mailbox));
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        self.add_reply_to(address);
        self
    }

    /// Adds a `Reply-To` header
    pub fn add_reply_to<A: IntoMailbox>(&mut self, address: A) {
        let mailbox = address.into_mailbox();
        self.reply_to_header.push(Address::Mailbox(mailbox));
    }

    /// Adds a `Sender` header
    pub fn sender<A: IntoMailbox>(mut self, address: A) -> EmailBuilder {
        self.set_sender(address);
        self
    }

    /// Adds a `Sender` header
    pub fn set_sender<A: IntoMailbox>(&mut self, address: A) {
        let mailbox = address.into_mailbox();
        self.sender_header = Some(mailbox);
    }

    /// Adds a `Subject` header
    pub fn subject<S: Into<String>>(mut self, subject: S) -> EmailBuilder {
        self.set_subject(subject);
        self
    }

    /// Adds a `Subject` header
    pub fn set_subject<S: Into<String>>(&mut self, subject: S) {
        self.message.add_header(("Subject".to_string(), subject.into()));
    }

    /// Adds a `Date` header with the given date
    pub fn date(mut self, date: &Tm) -> EmailBuilder {
        self.set_date(date);
        self
    }

    /// Adds a `Date` header with the given date
    pub fn set_date(&mut self, date: &Tm) {
        self.message.add_header(("Date", Tm::rfc822z(date).to_string()));
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
    pub fn text<S: Into<String>>(mut self, body: S) -> EmailBuilder {
        self.set_text(body);
        self
    }

    /// Sets the email body to plain text content
    pub fn set_text<S: Into<String>>(&mut self, body: S) {
        self.message.set_body(body);
        self.message
            .add_header(("Content-Type", format!("{}", mime!(Text/Plain; Charset=Utf8)).as_ref()));
    }

    /// Sets the email body to HTML content
    pub fn html<S: Into<String>>(mut self, body: S) -> EmailBuilder {
        self.set_html(body);
        self
    }

    /// Sets the email body to HTML content
    pub fn set_html<S: Into<String>>(&mut self, body: S) {
        self.message.set_body(body);
        self.message
            .add_header(("Content-Type", format!("{}", mime!(Text/Html; Charset=Utf8)).as_ref()));
    }

    /// Sets the email content
    pub fn alternative<S: Into<String>, T: Into<String>>(mut self,
                                                         body_html: S,
                                                         body_text: T)
                                                         -> EmailBuilder {
        self.set_alternative(body_html, body_text);
        self
    }

    /// Sets the email content
    pub fn set_alternative<S: Into<String>, T: Into<String>>(&mut self,
                                                             body_html: S,
                                                             body_text: T) {
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

    /// Sets the envelope for manual destination control
    /// If this function is not called, the envelope will be calculated
    /// from the "to" and "cc" addresses you set.
    pub fn envelope(mut self, envelope: Envelope) -> EmailBuilder {
        self.set_envelope(envelope);
        self
    }

    /// Sets the envelope for manual destination control
    /// If this function is not called, the envelope will be calculated
    /// from the "to" and "cc" addresses you set.
    pub fn set_envelope(&mut self, envelope: Envelope) {
        self.envelope = Some(envelope);
    }

    /// Builds the Email
    pub fn build(mut self) -> Result<Email, Error> {
        // If there are multiple addresses in "From", the "Sender" is required.
        if self.from_header.len() >= 2 && self.sender_header.is_none() {
            // So, we must find something to put as Sender.
            for possible_sender in self.from_header.iter() {
                // Only a mailbox can be used as sender, not Address::Group.
                if let &Address::Mailbox(ref mbx) = possible_sender {
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
            self.message.add_header(("Sender", v.to_string().as_ref()));
        }
        // Calculate the envelope
        let envelope = match self.envelope {
            Some(e) => e,
            None => {
                // we need to generate the envelope
                let mut e = Envelope::new();
                // add all receivers in to_header and cc_header
                for receiver in self.to_header.iter().chain(self.cc_header.iter()) {
                    match *receiver {
                        Address::Mailbox(ref m) => e.add_to(m.address.clone()),
                        Address::Group(_, ref ms) => {
                            for m in ms.iter() {
                                e.add_to(m.address.clone());
                            }
                        }
                    }
                }
                if e.to.is_empty() {
                    return Err(Error::MissingTo);
                }
                e.set_from(match self.sender_header {
                    Some(x) => x.address.clone(), // if we have a sender_header, use it
                    None => {
                        // use a from header
                        debug_assert!(self.from_header.len()<=1); // else we'd have sender_header
                        match self.from_header.first() {
                            Some(a) => match *a {
                                // if we have a from header
                                Address::Mailbox(ref mailbox) => mailbox.address.clone(), // use it
                                Address::Group(_,ref mailbox_list) => match mailbox_list.first() {
                                    // if it's an author group, use the first author
                                    Some(mailbox) => mailbox.address.clone(),
                                    // for an empty author group (the rarest of the rare cases)
                                    None => return Err(Error::MissingFrom), // empty envelope sender
                                },
                            },
                            // if we don't have a from header
                            None => return Err(Error::MissingFrom), // empty envelope sender
                        }
                    }
                });
                e
            }
        };
        // Add the collected addresses as mailbox-list all at once.
        // The unwraps are fine because the conversions for Vec<Address> never errs.
        if !self.to_header.is_empty() {
            self.message.add_header(Header::new_with_value("To".into(), self.to_header).unwrap());
        }
        if !self.from_header.is_empty() {
            self.message
                .add_header(Header::new_with_value("From".into(), self.from_header).unwrap());
        } else {
            return Err(Error::MissingFrom);
        }
        if !self.cc_header.is_empty() {
            self.message.add_header(Header::new_with_value("Cc".into(), self.cc_header).unwrap());
        }
        if !self.reply_to_header.is_empty() {
            self.message
                .add_header(Header::new_with_value("Reply-To".into(), self.reply_to_header)
                    .unwrap());
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
            envelope: envelope,
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
    /// Message ID
    fn message_id(&self) -> String;
    /// Message content
    fn message(self) -> String;
}

/// Minimal email structure
#[derive(Debug)]
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
    pub fn new(from_address: String,
               to_address: Vec<String>,
               message: String)
               -> SimpleSendableEmail {
        SimpleSendableEmail {
            from: from_address,
            to: to_address,
            message: message,
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

    fn message_id(&self) -> String {
        format!("{}", Uuid::new_v4())
    }

    fn message(self) -> String {
        self.message
    }
}

impl SendableEmail for Email {
    fn to_addresses(&self) -> Vec<String> {
        self.envelope.to.clone()
    }

    fn from_address(&self) -> String {
        self.envelope.from.clone()
    }

    fn message_id(&self) -> String {
        format!("{}", self.message_id)
    }

    fn message(self) -> String {
        self.to_string()
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
    fn test_multiple_from() {
        let email_builder = EmailBuilder::new();
        let date_now = now();
        let email = email_builder.to("anna@example.com")
            .from("dieter@example.com")
            .from("joachim@example.com")
            .date(&date_now)
            .subject("Invitation")
            .body("We invite you!")
            .build()
            .unwrap();
        assert_eq!(format!("{}", email),
                   format!("Date: {}\r\nSubject: Invitation\r\nSender: \
                            <dieter@example.com>\r\nTo: <anna@example.com>\r\nFrom: \
                            <dieter@example.com>, <joachim@example.com>\r\nMIME-Version: \
                            1.0\r\nMessage-ID: <{}.lettre@localhost>\r\n\r\nWe invite you!\r\n",
                           date_now.rfc822z(),
                           email.message_id()));
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
                   format!("Date: {}\r\nSubject: Hello\r\nX-test: value\r\nSender: \
                            <sender@localhost>\r\nTo: <user@localhost>\r\nFrom: \
                            <user@localhost>\r\nCc: \"Alias\" <cc@localhost>\r\nReply-To: \
                            <reply@localhost>\r\nMIME-Version: 1.0\r\nMessage-ID: \
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
        let content = format!("{}", email);
        assert_eq!(email.message(), content);
    }

}
