// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simple email (very incomplete)

use email::{MimeMessage, Header, Address};
use time::{now, Tm};
use uuid::Uuid;

use sendable_email::SendableEmail;

/// Converts an adress or an address with an alias to an `Address`
pub trait ToHeader {
    /// Converts to an `Address` struct
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

/// Converts an adress or an address with an alias to an `Address`
pub trait ToAddress {
    /// Converts to an `Address` struct
    fn to_address(&self) -> Address;
}

impl ToAddress for Address {
    fn to_address(&self) -> Address {
        (*self).clone()
    }
}

impl<'a> ToAddress for &'a str {
    fn to_address(&self) -> Address {
        Address::new_mailbox(self.to_string())
    }
}

impl<'a> ToAddress for (&'a str, &'a str) {
    fn to_address(&self) -> Address {
        let (address, alias) = *self;
        Address::new_mailbox_with_name(address.to_string(), alias.to_string())
    }
}

/// TODO
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

impl Email {
    /// Displays the formatted email content
    pub fn as_string(&self) -> String {
        self.message.as_string()
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

        email.message.headers.insert(
            Header::new_with_value("Message-ID".to_string(),
                format!("<{}@rust-smtp>", current_message)
            ).unwrap()
        );

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
    pub fn from<A: ToAddress>(mut self, address: A) -> EmailBuilder {
        self.content.from = Some(address.to_address().get_address().unwrap());
        self.insert_header(("From", address.to_address().to_string().as_ref()));
        self
    }

    /// Adds a `To` header and store the recipient address
    pub fn to<A: ToAddress>(mut self, address: A) -> EmailBuilder {
        self.content.to.push(address.to_address().get_address().unwrap());
        self.insert_header(("To", address.to_address().to_string().as_ref()));
        self
    }

    /// Adds a `Cc` header and store the recipient address
    pub fn cc<A: ToAddress>(mut self, address: A) -> EmailBuilder {
        self.content.to.push(address.to_address().get_address().unwrap());
        self.insert_header(("Cc", address.to_address().to_string().as_ref()));
        self
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: ToAddress>(mut self, address: A) -> EmailBuilder {
        self.insert_header(("Reply-To", address.to_address().to_string().as_ref()));
        self
    }

    /// Adds a `Sender` header
    pub fn sender<A: ToAddress>(mut self, address: A) -> EmailBuilder {
        self.content.from = Some(address.to_address().get_address().unwrap());
        self.insert_header(("Sender", address.to_address().to_string().as_ref()));
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
        format! ("{}", self.as_string())
    }

    fn message_id(&self) -> String {
        format!("{}", self.message_id)
    }
}
