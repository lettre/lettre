// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP response, containing a mandatory return code, and an optional text message

use std::str::FromStr;
use std::fmt::{Display, Formatter, Result};
use std::result::Result as RResult;

use tools::remove_trailing_crlf;

/// Contains an SMTP reply, with separed code and message
///
/// The text message is optional, only the code is mandatory
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct Response {
    /// Server response code
    pub code: u16,
    /// Server response string (optional)
    pub message: Option<String>
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write! (f, "{}",
            match self.clone().message {
                Some(message) => format!("{} {}", self.code, message),
                None => format!("{}", self.code),
            }
        )
    }
}

impl FromStr for Response {
    type Err = &'static str;
    fn from_str(s: &str) -> RResult<Response, &'static str> {
        // If the string is too short to be a response code
        if s.len() < 3 {
            Err("len < 3")
        // If we have only a code, with or without a trailing space
        } else if s.len() == 3 || (s.len() == 4 && &s[3..4] == " ") {
            match s[..3].parse::<u16>() {
                Ok(code) => Ok(Response{
                                code: code,
                                message: None
                              }),
                Err(_) => Err("Can't parse the code"),
            }
        // If we have a code and a message
        } else {
            match (
                s[..3].parse::<u16>(),
                vec![" ", "-"].contains(&&s[3..4]),
                (remove_trailing_crlf(&s[4..]))
            ) {
                (Ok(code), true, message) => Ok(Response{
                            code: code,
                            message: Some(message.to_string())
                        }),
                _ => Err("Error parsing a code with a message"),
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
            Ok(Response{
                code: 200,
                message: Some("response message".to_string())
            })
        );
        assert_eq!("200-response message".parse::<Response>(),
            Ok(Response{
                code: 200,
                message: Some("response message".to_string())
            })
        );
        assert_eq!("200".parse::<Response>(),
            Ok(Response{
                code: 200,
                message: None
            })
        );
        assert_eq!("200 ".parse::<Response>(),
            Ok(Response{
                code: 200,
                message: None
            })
        );
        assert_eq!("200-response\r\nmessage".parse::<Response>(),
            Ok(Response{
                code: 200,
                message: Some("response\r\nmessage".to_string())
            })
        );
        assert!("2000response message".parse::<Response>().is_err());
        assert!("20a response message".parse::<Response>().is_err());
        assert!("20 ".parse::<Response>().is_err());
        assert!("20".parse::<Response>().is_err());
        assert!("2".parse::<Response>().is_err());
        assert!("".parse::<Response>().is_err());
    }
}
