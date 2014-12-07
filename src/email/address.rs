// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simple SMTP "address" (very incomplete)

use std::fmt::{Show, Formatter, Result};

use common::SP;

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
        Address::new(*self, None)
    }
}

impl<'a> ToAddress for (&'a str, &'a str) {
    fn to_address(&self) -> Address {
        let (address, alias) = *self;
        Address::new(address, Some(alias))
    }
}

/// Contains an address with an optionnal alias
#[deriving(PartialEq,Eq,Clone)]
pub struct Address {
    /// The address
    address: String,
    /// The alias
    alias: Option<String>,
}

impl Show for Address {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write(match self.alias {
            Some(ref alias_string) => format!("{}{}<{}>", alias_string, SP, self.address.as_slice()),
            None => self.address.clone(),
        }.as_bytes())
    }
}

impl Address {
    /// Creates an address
    pub fn new(address: &str, alias: Option<&str>) -> Address {
        Address {
            address: address.to_string(),
            alias: match alias {
                Some(ref alias_string) => Some(alias_string.to_string()),
                None => None,
            }
        }
    }

    /// Return only the address
    pub fn get_address(&self) -> String {
        self.address.clone()
    }
}

#[cfg(test)]
mod test {
    use super::Address;

    #[test]
    fn test_new() {
        assert_eq!(
            Address::new("address", Some("alias")),
            Address{address: "address".to_string(), alias: Some("alias".to_string())}
        );
        assert_eq!(
            Address::new("address", None),
            Address{address: "address".to_string(), alias: None}
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!(
            format!("{}", Address::new("address", None)),
            "address".to_string()
        );
        assert_eq!(
            format!("{}", Address::new("address", Some("alias"))),
            "alias <address>".to_string()
        );
    }
}
