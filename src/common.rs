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
    match address.len() {
        0..1 => format!("<{:s}>", address),
        _   => match (address.slice_to(1), address.slice_from(address.len() - 1)) {
                   ("<", ">") => address,
                   _          => format!("<{:s}>", address)
               }
    }
}

/// Removes quotes from emails if needed
pub fn unquote_email_address(address: ~str) -> ~str {
    match address.len() {
        0..1 => address,
        _    => match (address.slice_to(1), address.slice_from(address.len() - 1)) {
                    ("<", ">") => address.slice(1, address.len() - 1).to_owned(),
                    _          => address
                }
    }
}

/// Removes the trailing line return at the end of a string
pub fn remove_trailing_crlf(string: ~str) -> ~str {
    if string.len() > 1 && string.slice_from(string.len() - 2) == CRLF {
        string.slice_to(string.len() - 2).to_owned()
    } else if string.len() > 0 && string.slice_from(string.len() - 1) == "\r" {
        string.slice_to(string.len() - 1).to_owned()
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
        assert_eq!(super::quote_email_address("a".to_owned()), "<a>".to_owned());
        assert_eq!(super::quote_email_address("".to_owned()), "<>".to_owned());
    }

    #[test]
    fn test_unquote_email_address() {
        assert_eq!(super::unquote_email_address("<address>".to_owned()), "address".to_owned());
        assert_eq!(super::unquote_email_address("address".to_owned()), "address".to_owned());
        assert_eq!(super::unquote_email_address("<address".to_owned()), "<address".to_owned());
        assert_eq!(super::unquote_email_address("<>".to_owned()), "".to_owned());
        assert_eq!(super::unquote_email_address("a".to_owned()), "a".to_owned());
        assert_eq!(super::unquote_email_address("".to_owned()), "".to_owned());
    }

    #[test]
    fn test_remove_trailing_crlf() {
        assert_eq!(super::remove_trailing_crlf("word".to_owned()), "word".to_owned());
        assert_eq!(super::remove_trailing_crlf("word\r\n".to_owned()), "word".to_owned());
        assert_eq!(super::remove_trailing_crlf("word\r\n ".to_owned()), "word\r\n ".to_owned());
        assert_eq!(super::remove_trailing_crlf("word\r".to_owned()), "word".to_owned());
        assert_eq!(super::remove_trailing_crlf("\r\n".to_owned()), "".to_owned());
        assert_eq!(super::remove_trailing_crlf("\r".to_owned()), "".to_owned());
        assert_eq!(super::remove_trailing_crlf("a".to_owned()), "a".to_owned());
        assert_eq!(super::remove_trailing_crlf("".to_owned()), "".to_owned());
    }

    #[test]
    fn test_get_first_word() {
        assert_eq!(super::get_first_word("first word".to_owned()), "first".to_owned());
        assert_eq!(super::get_first_word("first word\r\ntest".to_owned()), "first".to_owned());
        assert_eq!(super::get_first_word("first".to_owned()), "first".to_owned());
        assert_eq!(super::get_first_word("".to_owned()), "".to_owned());
        assert_eq!(super::get_first_word("\r\n".to_owned()), "".to_owned());
        assert_eq!(super::get_first_word("a\r\n".to_owned()), "a".to_owned());
        // Manage cases of empty line, spaces at the beginning, ...
        //assert_eq!(super::get_first_word(" a".to_owned()), "a".to_owned());
        //assert_eq!(super::get_first_word("\r\n a".to_owned()), "a".to_owned());
        assert_eq!(super::get_first_word(" \r\n".to_owned()), "".to_owned());
        assert_eq!(super::get_first_word("\r\n ".to_owned()), "".to_owned());
    }
}
