//! Simple email (very incomplete)

use std::fmt;
use std::fmt::{Display, Formatter};

use email_format::{Header, Mailbox, MimeMessage, MimeMultipartType};
use mime::Mime;
use time::{Tm, now};
use uuid::Uuid;

/// Insert a header in a message
fn insert_header<A: ToHeader>(message: &mut MimeMessage, header: A) {
    message.headers.insert(header.to_header());
}

/// Converts an adress or an address with an alias to a `Address`
pub trait ToHeader {
    /// Converts to a `Header` struct
    fn to_header(&self) -> Header;
}

impl ToHeader for Header {
    fn to_header(&self) -> Header {
        (*self).clone()
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

/// Builds an `Email` structure
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct EmailBuilder {
    /// Message
    message: MimeMessage,
    /// The enveloppe recipients addresses
    to: Vec<String>,
    /// The enveloppe sender address
    from: Option<String>,
    /// Date issued
    date_issued: bool,
}

/// Simple email representation
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct Email {
    /// Message
    message: MimeMessage,
    /// The enveloppe recipients addresses
    to: Vec<String>,
    /// The enveloppe sender address
    from: String,
    /// Message-ID
    message_id: Uuid,
}

impl Display for Email {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.message.as_string())
    }
}

impl EmailBuilder {
    /// Creates a new empty email
    pub fn new() -> EmailBuilder {
        EmailBuilder {
            message: MimeMessage::new_blank_message(),
            to: vec![],
            from: None,
            date_issued: false,
        }
    }

    /// Sets the email body
    pub fn body(mut self, body: &str) -> EmailBuilder {
        self.message.body = body.to_string();
        self
    }

    /// Add a generic header
    pub fn add_header<A: ToHeader>(mut self, header: A) -> EmailBuilder {
        insert_header(&mut self.message, header);
        self
    }

    /// Adds a `From` header and store the sender address
    pub fn from<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.to_mailbox();
        insert_header(&mut self.message, ("From", mailbox.to_string().as_ref()));
        self.from = Some(mailbox.address);
        self
    }

    /// Adds a `To` header and store the recipient address
    pub fn to<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.to_mailbox();
        insert_header(&mut self.message, ("To", mailbox.to_string().as_ref()));
        self.to.push(mailbox.address);
        self
    }

    /// Adds a `Cc` header and store the recipient address
    pub fn cc<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.to_mailbox();
        insert_header(&mut self.message, ("Cc", mailbox.to_string().as_ref()));
        self.to.push(mailbox.address);
        self
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.to_mailbox();
        insert_header(&mut self.message,
                      ("Reply-To", mailbox.to_string().as_ref()));
        self
    }

    /// Adds a `Sender` header
    pub fn sender<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.to_mailbox();
        insert_header(&mut self.message, ("Sender", mailbox.to_string().as_ref()));
        self.from = Some(mailbox.address);
        self
    }

    /// Adds a `Subject` header
    pub fn subject(mut self, subject: &str) -> EmailBuilder {
        insert_header(&mut self.message, ("Subject", subject));
        self
    }

    /// Adds a `Date` header with the given date
    pub fn date(mut self, date: &Tm) -> EmailBuilder {
        insert_header(&mut self.message,
                      ("Date", Tm::rfc822z(date).to_string().as_ref()));
        self.date_issued = true;
        self
    }

    /// Adds a `ContentType` header with the given MIME type
    pub fn content_type(mut self, content_type: Mime) -> EmailBuilder {
        insert_header(&mut self.message,
                      ("Content-Type", format!("{}", content_type).as_ref()));
        self
    }

    /// Sets the email body to a plain text content
    pub fn text(mut self, body: &str) -> EmailBuilder {
        self.message.body = body.to_string();
        insert_header(&mut self.message,
                      ("Content-Type", format!("{}", mime!(Text/Plain; Charset=Utf8)).as_ref()));
        self
    }

    /// Sets the email body to a HTML contect
    pub fn html(mut self, body: &str) -> EmailBuilder {
        self.message.body = body.to_string();
        insert_header(&mut self.message,
                      ("Content-Type", format!("{}", mime!(Text/Html; Charset=Utf8)).as_ref()));
        self
    }

    /// Sets the email content
    pub fn alternative(mut self, body_html: &str, body_text: &str) -> EmailBuilder {
        let mut alternate = MimeMessage::new_blank_message();
        alternate.message_type = Some(MimeMultipartType::Alternative);

        let mut text = MimeMessage::new(body_text.to_string());
        insert_header(&mut text,
                      ("Content-Type", format!("{}", mime!(Text/Plain; Charset=Utf8)).as_ref()));
        text.update_headers();

        let mut html = MimeMessage::new(body_html.to_string());
        insert_header(&mut html,
                      ("Content-Type", format!("{}", mime!(Text/Html; Charset=Utf8)).as_ref()));
        html.update_headers();

        alternate.children.push(text);
        alternate.children.push(html);
        alternate.update_headers();

        self.message.message_type = Some(MimeMultipartType::Mixed);
        self.message.children.push(alternate);

        self
    }

    /// Build the Email
    pub fn build(mut self) -> Result<Email, &'static str> {
        if self.from.is_none() {
            return Err("No from address");
        }
        if self.to.is_empty() {
            return Err("No to address");
        }

        if !self.date_issued {
            insert_header(&mut self.message,
                          ("Date", Tm::rfc822z(&now()).to_string().as_ref()));
        }

        let message_id = Uuid::new_v4();

        match Header::new_with_value("Message-ID".to_string(),
                                     format!("<{}.lettre@localhost>", message_id)) {
            Ok(header) => insert_header(&mut self.message, header),
            Err(_) => (),
        }

        self.message.update_headers();

        Ok(Email {
            message: self.message,
            to: self.to,
            from: self.from.unwrap(),
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
        self.to.clone()
    }

    fn from_address(&self) -> String {
        self.from.clone()
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
    use time::now;

    use uuid::Uuid;
    use email_format::{Header, MimeMessage};

    use super::{Email, EmailBuilder, SendableEmail};

    #[test]
    fn test_email_display() {
        let current_message = Uuid::new_v4();

        let mut email = Email {
            message: MimeMessage::new_blank_message(),
            to: vec![],
            from: "".to_string(),
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
                                 .add_header(("X-test", "value"))
                                 .build()
                                 .unwrap();

        assert_eq!(format!("{}", email),
                   format!("To: <user@localhost>\r\nFrom: <user@localhost>\r\nCc: \"Alias\" \
                            <cc@localhost>\r\nReply-To: <reply@localhost>\r\nSender: \
                            <sender@localhost>\r\nDate: {}\r\nSubject: Hello\r\nX-test: \
                            value\r\nMessage-ID: <{}.lettre@localhost>\r\n\r\nHello World!\r\n",
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
                                 .add_header(("X-test", "value"))
                                 .build()
                                 .unwrap();

        assert_eq!(email.from_address(), "sender@localhost".to_string());
        assert_eq!(email.to_addresses(),
                   vec!["user@localhost".to_string(), "cc@localhost".to_string()]);
        assert_eq!(email.message(), format!("{}", email));
    }

}
