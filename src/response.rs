// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP response, containing a mandatory return code, and an optional text message

#![unstable]

use std::str::FromStr;
use std::fmt::{Show, Formatter, Result};

use tools::remove_trailing_crlf;

/// Contains an SMTP reply, with separed code and message
///
/// The text message is optional, only the code is mandatory
#[deriving(PartialEq,Eq,Clone)]
pub struct Response {
    /// Server response code
    pub code: u16,
    /// Server response string (optional)
    pub message: Option<String>
}

impl Show for Response {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write(
            match self.clone().message {
                Some(message) => format!("{} {}", self.code, message),
                None => format!("{}", self.code),
            }.as_bytes()
        )
    }
}

impl FromStr for Response {
    fn from_str(s: &str) -> Option<Response> {
        // If the string is too short to be a response code
        if s.len() < 3 {
            None
        // If we have only a code, with or without a trailing space
        } else if s.len() == 3 || (s.len() == 4 && s.slice(3,4) == " ") {
            match s.slice_to(3).parse::<u16>() {
                Some(code) => Some(Response{
                                code: code,
                                message: None
                              }),
                None => None,
            }
        // If we have a code and a message
        } else {
            match (
                s.slice_to(3).parse::<u16>(),
                vec![" ", "-"].contains(&s.slice(3,4)),
                (remove_trailing_crlf(s.slice_from(4)))
            ) {
                (Some(code), true, message) => Some(Response{
                            code: code,
                            message: Some(message.to_string())
                        }),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Response;

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", Response{code: 200, message: Some("message".to_string())}),
                   "200 message".to_string());
    }

    #[test]
    fn test_from_str() {
        assert_eq!("200 response message".parse::<Response>(),
            Some(Response{
                code: 200,
                message: Some("response message".to_string())
            })
        );
        assert_eq!("200-response message".parse::<Response>(),
            Some(Response{
                code: 200,
                message: Some("response message".to_string())
            })
        );
        assert_eq!("200".parse::<Response>(),
            Some(Response{
                code: 200,
                message: None
            })
        );
        assert_eq!("200 ".parse::<Response>(),
            Some(Response{
                code: 200,
                message: None
            })
        );
        assert_eq!("200-response\r\nmessage".parse::<Response>(),
            Some(Response{
                code: 200,
                message: Some("response\r\nmessage".to_string())
            })
        );
        assert_eq!("2000response message".parse::<Response>(), None);
        assert_eq!("20a response message".parse::<Response>(), None);
        assert_eq!("20 ".parse::<Response>(), None);
        assert_eq!("20".parse::<Response>(), None);
        assert_eq!("2".parse::<Response>(), None);
        assert_eq!("".parse::<Response>(), None);
    }
}
