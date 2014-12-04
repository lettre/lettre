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

/// TODO
#[deriving(PartialEq,Eq,Clone)]
pub struct Header {
    /// TODO
    name: String,
    /// TODO
    value: String,
}

impl Show for Header {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write(format!("{}{}{}{}", self.name, COLON, SP, self.value).as_bytes())
    }
}

impl Header {
    /// TODO
    pub fn new(name: &str, value: &str) -> Header {
        Header{name: name.to_string(), value: value.to_string()}
    }
}

// impl Str for Header {
//     fn as_slice<'a>(&'a self) -> &'a str {
//         self.clone().to_string().clone().as_slice()
//     }
// }

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
