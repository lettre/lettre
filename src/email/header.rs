// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simple SMTP headers

use std::fmt::{Show, Formatter, Result};

use common::{SP, COLON};

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
#[deriving(PartialEq,Eq,Clone)]
pub struct Header {
    /// Name of the header
    name: String,
    /// Value of the header
    value: String,
}

impl Show for Header {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write(format!("{}{}{}{}", self.name, COLON, SP, self.value).as_bytes())
    }
}

impl Header {
    /// Creates ah `Header`
    pub fn new(name: &str, value: &str) -> Header {
        Header{name: name.to_string(), value: value.to_string()}
    }
}

#[cfg(test)]
mod test {
    use super::Header;

    #[test]
    fn test_new() {
        assert_eq!(
            Header::new("From", "me"),
            Header{name: "From".to_string(), value: "me".to_string()}
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!(
            format!("{}", Header::new("From", "me")),
            "From: me".to_string()
        );
    }
}
