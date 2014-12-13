// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP sendable email

/// Email sendable by an SMTP client
pub trait SendableEmail {
    /// From address
    fn from_address(&self) -> String;
    /// To addresses
    fn to_addresses(&self) -> Vec<String>;
    /// Message content
    fn message(&self) -> String;
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
    pub fn new(from_address: &str, to_address: &str, message: &str) -> SimpleSendableEmail {
        SimpleSendableEmail {
            from: from_address.to_string(),
            to: vec!(to_address.to_string()),
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
}
