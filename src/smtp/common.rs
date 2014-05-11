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
//! Needs to be organized later

pub static SP: &'static str = " ";
pub static CRLF: &'static str = "\r\n";

/// Adds quotes to emails if needed
pub fn quote_email_address(address: ~str) -> ~str {
    match (address.as_slice().slice_to(1), address.as_slice().slice_from(address.as_slice().len()-1)) {
        ("<", ">") => address,
        _          => format!("<{:s}>", address)
    }
}

/// Removes quotes from emails if needed
pub fn unquote_email_address(address: ~str) -> ~str {
    match (address.as_slice().slice_to(1), address.as_slice().slice_from(address.as_slice().len() - 1)) {
        ("<", ">") => address.as_slice().slice(1, address.as_slice().len() - 1).to_owned(),
        _          => address
    }
}

/// Removes the trailing line return at the end of a string
pub fn remove_trailing_crlf(string: ~str) -> ~str {
    if string.as_slice().slice_from(string.as_slice().len() - 2) == CRLF {
        string.as_slice().slice_to(string.as_slice().len() - 2).to_owned()
    } else if string.as_slice().slice_from(string.as_slice().len() - 1) == "\r" {
        string.as_slice().slice_to(string.as_slice().len() - 1).to_owned()
    } else {
        string
    }
}

/// Returns the first word of a string, or the string if it contains no space
pub fn get_first_word(string: ~str) -> ~str {
    string.split_str(CRLF).next().unwrap().splitn(' ', 1).next().unwrap().to_owned()
}

#[cfg(test)]
mod test {
    #[test]
    fn test_quote_email_address() {
        assert_eq!(super::quote_email_address("address".to_owned()), "<address>".to_owned());
        assert_eq!(super::quote_email_address("<address>".to_owned()), "<address>".to_owned());
    }

    #[test]
    fn test_unquote_email_address() {
        assert_eq!(super::unquote_email_address("<address>".to_owned()), "address".to_owned());
        assert_eq!(super::unquote_email_address("address".to_owned()), "address".to_owned());
        assert_eq!(super::unquote_email_address("<address".to_owned()), "<address".to_owned());
    }

    #[test]
    fn test_remove_trailing_crlf() {
        assert_eq!(super::remove_trailing_crlf("word".to_owned()), "word".to_owned());
        assert_eq!(super::remove_trailing_crlf("word\r\n".to_owned()), "word".to_owned());
        assert_eq!(super::remove_trailing_crlf("word\r\n ".to_owned()), "word\r\n ".to_owned());
        assert_eq!(super::remove_trailing_crlf("word\r".to_owned()), "word".to_owned());
    }

    #[test]
    fn test_get_first_word() {
        assert_eq!(super::get_first_word("first word".to_owned()), "first".to_owned());
        assert_eq!(super::get_first_word("first word\r\ntest".to_owned()), "first".to_owned());
        assert_eq!(super::get_first_word("first".to_owned()), "first".to_owned());
    }
}
