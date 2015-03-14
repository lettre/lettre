// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simple SMTP headers

use time::Tm;

use std::fmt::{Display, Formatter, Result};

use mailer::address::Address;
use {COLON, SP};

/// Converts to an `Header`
pub trait ToHeader {
    /// Converts to an `Header` struct
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
        Header::new(name, value)
    }
}

/// Contains a header
#[derive(PartialEq,Eq,Clone,Debug)]
pub enum Header {
    /// `To`
    To(Address),
    /// `From`
    From(Address),
    /// `Cc`
    Cc(Address),
    /// `Reply-To`
    ReplyTo(Address),
    /// `Sender`
    Sender(Address),
    /// `Date`
    Date(Tm),
    /// `Subject`
    Subject(String),
    /// `MIME-Version`
    MimeVersion,
    /// `Content-Type`
    ContentType(String),
    /// `Message-Id`
    MessageId(String),
    /// Any header (name, value)
    Other(String, String),
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write! (f, "{}{}{}{}",
        match *self {
            Header::To(_) => "To",
            Header::From(_) => "From",
            Header::Cc(_) => "Cc",
            Header::ReplyTo(_) => "Reply-To",
            Header::Sender(_) => "Sender",
            Header::Date(_) => "Date",
            Header::Subject(_) => "Subject",
            Header::MimeVersion => "MIME-Version",
            Header::ContentType(_) => "Content-Type",
            Header::MessageId(_) => "Message-Id",
            Header::Other(ref name, _) => name.as_slice(),
        },
        COLON, SP,
        match *self {
            Header::To(ref address) => format! ("{}", address),
            Header::From(ref address) => format! ("{}", address),
            Header::Cc(ref address) => format! ("{}", address),
            Header::ReplyTo(ref address) => format! ("{}", address),
            Header::Sender(ref address) => format! ("{}", address),
            Header::Date(ref date) => Tm::rfc822(date).to_string(),
            Header::Subject(ref subject) => subject.clone(),
            Header::MimeVersion => "1.0".to_string(),
            Header::ContentType(ref string) => string.clone(),
            Header::MessageId(ref string) => string.clone(),
            Header::Other(_, ref value) => value.clone(),
        })
    }
}

impl Header {
    /// Creates ah `Header`
    pub fn new(name: &str, value: &str) -> Header {
        Header::Other(name.to_string(), value.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::Header;
    use mailer::address::Address;

    use time::{at_utc, Timespec};

    #[test]
    fn test_new() {
        assert_eq!(
            Header::new("From", "me"),
            Header::Other("From".to_string(), "me".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!(
            format!("{}", Header::new("From", "me")),
            "From: me".to_string()
        );
        assert_eq!(
            format!("{}", Header::To(Address::new("me@example.com", Some("My Name")))),
            "To: My Name <me@example.com>".to_string()
        );
        assert_eq!(
            format!("{}", Header::Subject("Test subject".to_string())),
            "Subject: Test subject".to_string()
        );
        let time = at_utc(Timespec::new(1234567890, 54321));
        assert_eq!(
            format!("{}", Header::Date(time)),
            "Date: Fri, 13 Feb 2009 23:31:30 GMT".to_string()
        );
    }
}
