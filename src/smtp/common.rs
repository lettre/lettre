// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Common definitions for SMTP
//!
//! Needs to be organized later.
 
use std::strbuf::StrBuf;

pub static SP: &'static str = " ";
pub static CRLF: &'static str = "\r\n";

/// Adds quotes to emails if needed
pub fn quote_email_address(addr: &str) -> ~str {
    match (addr.slice_to(1), addr.slice_from(addr.len()-1)) {
        ("<", ">") => addr.to_owned(),
        _          => format!("<{:s}>", addr)
    }
}

/// Removes quotes from emails if needed
pub fn unquote_email_address(addr: &str) -> ~str {
    match (addr.slice_to(1), addr.slice_from(addr.len() - 1)) {
        ("<", ">") => addr.slice(1, addr.len() - 1).to_owned(),
        _          => addr.to_owned()
    }
}

/// Returns the first word of a string, or the string if it contains no space
pub fn get_first_word<T: Str>(string: T) -> StrBuf {
    StrBuf::from_str(string.into_owned().split_str(CRLF).next().unwrap().splitn(' ', 1).next().unwrap())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_quote_email_address() {
        assert!(super::quote_email_address("plop") == ~"<plop>");
        assert!(super::quote_email_address("<plop>") == ~"<plop>");
    }

    #[test]
    fn test_unquote_email_address() {
        assert!(super::unquote_email_address("<plop>") == ~"plop");
        assert!(super::unquote_email_address("plop") == ~"plop");
        assert!(super::unquote_email_address("<plop") == ~"<plop");
    }

    #[test]
    fn test_get_first_word() {
        assert!(super::get_first_word("first word") == StrBuf::from_str("first"));
        assert!(super::get_first_word("first word\r\ntest") == StrBuf::from_str("first"));
        assert!(super::get_first_word("first") == StrBuf::from_str("first"));
    }
}
