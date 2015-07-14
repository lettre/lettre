// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simple email (very incomplete)

use std::fmt::{Display, Formatter, Result};

use email_format::{MimeMessage, Header, Mailbox};
use time::{now, Tm};
use uuid::Uuid;

use sendable_email::SendableEmail;

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
    /// Email content
    content: Email,
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
    from: Option<String>,
    /// Message-ID
    message_id: Uuid,
}

impl Display for Email {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}",
            self.message.as_string()
        )
    }
}

impl EmailBuilder {
    /// Creates a new empty email
    pub fn new() -> EmailBuilder {
        let current_message = Uuid::new_v4();

        let mut email = Email {
            message: MimeMessage::new_blank_message(),
            to: vec![],
            from: None,
            message_id: current_message,
        };

        match Header::new_with_value("Message-ID".to_string(), format!("<{}@rust-smtp>", current_message)) {
            Ok(header) => email.message.headers.insert(header),
            Err(_) => (),
        }

        EmailBuilder {
            content: email,
            date_issued: false,
        }
    }

    /// Sets the email body
    pub fn body(mut self, body: &str) -> EmailBuilder {
        self.content.message.body = body.to_string();
        self
    }

    /// Add a generic header
    pub fn add_header<A: ToHeader>(mut self, header: A) -> EmailBuilder {
        self.insert_header(header);
        self
    }

    fn insert_header<A: ToHeader>(&mut self, header: A) {
        self.content.message.headers.insert(header.to_header());
    }

    /// Adds a `From` header and store the sender address
    pub fn from<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.to_mailbox();
        self.insert_header(("From", mailbox.to_string().as_ref()));
        self.content.from = Some(mailbox.address);
        self
    }

    /// Adds a `To` header and store the recipient address
    pub fn to<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.to_mailbox();
        self.insert_header(("To", mailbox.to_string().as_ref()));
        self.content.to.push(mailbox.address);
        self
    }

    /// Adds a `Cc` header and store the recipient address
    pub fn cc<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.to_mailbox();
        self.insert_header(("Cc", mailbox.to_string().as_ref()));
        self.content.to.push(mailbox.address);
        self
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.to_mailbox();
        self.insert_header(("Reply-To", mailbox.to_string().as_ref()));
        self
    }

    /// Adds a `Sender` header
    pub fn sender<A: ToMailbox>(mut self, address: A) -> EmailBuilder {
        let mailbox = address.to_mailbox();
        self.insert_header(("Sender", mailbox.to_string().as_ref()));
        self.content.from = Some(mailbox.address);
        self
    }

    /// Adds a `Subject` header
    pub fn subject(mut self, subject: &str) -> EmailBuilder {
        self.insert_header(("Subject", subject));
        self
    }

    /// Adds a `Date` header with the given date
    pub fn date(mut self, date: &Tm) -> EmailBuilder {
        self.insert_header(("Date", Tm::rfc822(date).to_string().as_ref()));
        self.date_issued = true;
        self
    }

    /// Build the Email
    pub fn build(mut self) -> Email {
        if !self.date_issued {
            self.insert_header(("Date", Tm::rfc822(&now()).to_string().as_ref()));
        }
        self.content.message.update_headers();
        self.content
    }
}

impl SendableEmail for Email {
    /// Return the to addresses, and fails if it is not set
    fn to_addresses(&self) -> Vec<String> {
        if self.to.is_empty() {
            panic!("The To field is empty")
        }
        self.to.clone()
    }

    /// Return the from address, and fails if it is not set
    fn from_address(&self) -> String {
        match self.from {
            Some(ref from_address) => from_address.clone(),
            None => panic!("The From field is empty"),
        }
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
    use email_format::{MimeMessage, Header};

    use sendable_email::SendableEmail;
    use super::{EmailBuilder, Email};

    #[test]
    fn test_email_display() {
        let current_message = Uuid::new_v4();

        let mut email = Email {
            message: MimeMessage::new_blank_message(),
            to: vec![],
            from: None,
            message_id: current_message,
        };

        email.message.headers.insert(
            Header::new_with_value("Message-ID".to_string(),
                format!("<{}@rust-smtp>", current_message)
            ).unwrap()
        );

        email.message.headers.insert(
            Header::new_with_value("To".to_string(), "to@example.com".to_string()).unwrap()
        );

        email.message.body = "body".to_string();

        assert_eq!(
            format!("{}", email),
            format!("Message-ID: <{}@rust-smtp>\r\nTo: to@example.com\r\n\r\nbody\r\n", current_message)
        );
        assert_eq!(current_message.to_string(), email.message_id());
    }

    #[test]
    fn test_email_builder() {
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
                             .build();

        assert_eq!(
            format!("{}", email),
            format!("Message-ID: <{}@rust-smtp>\r\nTo: <user@localhost>\r\nFrom: <user@localhost>\r\nCc: \"Alias\" <cc@localhost>\r\nReply-To: <reply@localhost>\r\nSender: <sender@localhost>\r\nDate: {}\r\nSubject: Hello\r\nX-test: value\r\n\r\nHello World!\r\n",
                    email.message_id(), date_now.rfc822())
        );
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
                             .build();

        assert_eq!(
            email.from_address(),
            "sender@localhost".to_string()
        );
        assert_eq!(
            email.to_addresses(),
            vec!["user@localhost".to_string(), "cc@localhost".to_string()]
        );
        assert_eq!(
            email.message(),
            format!("{}", email)
        );
    }

}
