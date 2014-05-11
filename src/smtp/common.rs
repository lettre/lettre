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

use std::strbuf::StrBuf;

pub static SP: &'static str = " ";
pub static CRLF: &'static str = "\r\n";

/// Adds quotes to emails if needed
pub fn quote_email_address(address: StrBuf) -> StrBuf {
    match (address.as_slice().slice_to(1), address.as_slice().slice_from(address.as_slice().len()-1)) {
        ("<", ">") => address.into_strbuf(),
        _          => StrBuf::from_str(format!("<{:s}>", address))
    }
}

/// Removes quotes from emails if needed
pub fn unquote_email_address(address: StrBuf) -> StrBuf {
    match (address.as_slice().slice_to(1), address.as_slice().slice_from(address.as_slice().len() - 1)) {
        ("<", ">") => address.as_slice().slice(1, address.as_slice().len() - 1).into_strbuf(),
        _          => address.into_strbuf()
    }
}

/// Removes the trailing line return at the end of a string
pub fn remove_trailing_crlf(string: StrBuf) -> StrBuf {
    if string.as_slice().slice_from(string.as_slice().len() - 2) == CRLF {
        StrBuf::from_str(string.as_slice().slice_to(string.as_slice().len() - 2))
    } else if string.as_slice().slice_from(string.as_slice().len() - 1) == "\r" {
        StrBuf::from_str(string.as_slice().slice_to(string.as_slice().len() - 1))
    } else {
        StrBuf::from_str(string.as_slice())
    }
}

/// Returns the first word of a string, or the string if it contains no space
pub fn get_first_word(string: StrBuf) -> StrBuf {
    StrBuf::from_str(string.into_owned().split_str(CRLF).next().unwrap().splitn(' ', 1).next().unwrap())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_quote_email_address() {
        assert_eq!(super::quote_email_address(StrBuf::from_str("plop")), StrBuf::from_str("<plop>"));
        assert_eq!(super::quote_email_address(StrBuf::from_str("<plop>")), StrBuf::from_str("<plop>"));
    }

    #[test]
    fn test_unquote_email_address() {
        assert_eq!(super::unquote_email_address(StrBuf::from_str("<plop>")), StrBuf::from_str("plop"));
        assert_eq!(super::unquote_email_address(StrBuf::from_str("plop")), StrBuf::from_str("plop"));
        assert_eq!(super::unquote_email_address(StrBuf::from_str("<plop")), StrBuf::from_str("<plop"));
    }

    #[test]
    fn test_remove_trailing_crlf() {
        assert_eq!(super::remove_trailing_crlf(StrBuf::from_str("word")), StrBuf::from_str("word"));
        assert_eq!(super::remove_trailing_crlf(StrBuf::from_str("word\r\n")), StrBuf::from_str("word"));
        assert_eq!(super::remove_trailing_crlf(StrBuf::from_str("word\r\n ")), StrBuf::from_str("word\r\n "));
        assert_eq!(super::remove_trailing_crlf(StrBuf::from_str("word\r")), StrBuf::from_str("word"));
    }

    #[test]
    fn test_get_first_word() {
        assert_eq!(super::get_first_word(StrBuf::from_str("first word")), StrBuf::from_str("first"));
        assert_eq!(super::get_first_word(StrBuf::from_str("first word\r\ntest")), StrBuf::from_str("first"));
        assert_eq!(super::get_first_word(StrBuf::from_str("first")), StrBuf::from_str("first"));
    }
}
