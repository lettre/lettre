// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tools for common string manipulations

use std::string::String;

/// The word separator for SMTP transactions
pub static SP: &'static str = " ";

/// The line ending for SMTP transactions (carriage return + line feed)
pub static CRLF: &'static str = "\r\n";

/// Carriage return
pub static CR: &'static str = "\r";

/// Line feed
pub static LF: &'static str = "\n";

/// Colon
pub static COLON: &'static str = ":";

/// The ending of message content
pub static MESSAGE_ENDING: &'static str = "\r\n.\r\n";

/// NUL unicode character
pub static NUL: &'static str = "\0";

/// Removes the trailing line return at the end of a string
#[inline]
pub fn remove_trailing_crlf(string: &str) -> &str {
    if string.ends_with(CRLF) {
        &string[.. string.len() - 2]
    } else if string.ends_with(CR) {
        &string[.. string.len() - 1]
    } else {
        string
    }
}

/// Returns the first word of a string, or the string if it contains no space
#[inline]
pub fn get_first_word(string: &str) -> &str {
    string.split(CRLF).next().unwrap().splitn(1, ' ').next().unwrap()
}

/// Returns the string replacing all the CRLF with "\<CRLF\>"
#[inline]
pub fn escape_crlf(string: &str) -> String {
    string.replace(CRLF, "<CR><LF>")
}

/// Returns the string after adding a dot at the beginning of each line starting with a dot
///
/// Reference : https://tools.ietf.org/html/rfc5321#page-62 (4.5.2. Transparency)
#[inline]
pub fn escape_dot(string: &str) -> String {
    if string.starts_with(".") {
        format!(".{}", string)
    } else {
        string.to_string()
    }.replace(format!("{}.", CR).as_slice(), format!("{}..", CR).as_slice())
     .replace(format!("{}.", LF).as_slice(), format!("{}..", LF).as_slice())
}

#[cfg(test)]
mod test {
    use super::{remove_trailing_crlf, get_first_word, escape_crlf, escape_dot};

    #[test]
    fn test_remove_trailing_crlf() {
        assert_eq!(remove_trailing_crlf("word"), "word");
        assert_eq!(remove_trailing_crlf("word\r\n"), "word");
        assert_eq!(remove_trailing_crlf("word\r\n "), "word\r\n ");
        assert_eq!(remove_trailing_crlf("word\r"), "word");
        assert_eq!(remove_trailing_crlf("\r\n"), "");
        assert_eq!(remove_trailing_crlf("\r"), "");
        assert_eq!(remove_trailing_crlf("a"), "a");
        assert_eq!(remove_trailing_crlf(""), "");
    }

    #[test]
    fn test_get_first_word() {
        assert_eq!(get_first_word("first word"), "first");
        assert_eq!(get_first_word("first word\r\ntest"), "first");
        assert_eq!(get_first_word("first"), "first");
        assert_eq!(get_first_word(""), "");
        assert_eq!(get_first_word("\r\n"), "");
        assert_eq!(get_first_word("a\r\n"), "a");
        // Manage cases of empty line, spaces at the beginning
        //assert_eq!(get_first_word(" a"), "a");
        //assert_eq!(get_first_word("\r\n a"), "a");
        assert_eq!(get_first_word(" \r\n"), "");
        assert_eq!(get_first_word("\r\n "), "");
    }

    #[test]
    fn test_escape_crlf() {
        assert_eq!(escape_crlf("\r\n").as_slice(), "<CR><LF>");
        assert_eq!(escape_crlf("EHLO my_name\r\n").as_slice(), "EHLO my_name<CR><LF>");
        assert_eq!(
            escape_crlf("EHLO my_name\r\nSIZE 42\r\n").as_slice(),
            "EHLO my_name<CR><LF>SIZE 42<CR><LF>"
        );
    }

    #[test]
    fn test_escape_dot() {
        assert_eq!(escape_dot(".test").as_slice(), "..test");
        assert_eq!(escape_dot("\r.\n.\r\n").as_slice(), "\r..\n..\r\n");
        assert_eq!(escape_dot("test\r\n.test\r\n").as_slice(), "test\r\n..test\r\n");
        assert_eq!(escape_dot("test\r\n.\r\ntest").as_slice(), "test\r\n..\r\ntest");
    }
}
