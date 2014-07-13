// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP responses, contaiing a mandatory return code, and an optional text message

use std::from_str::FromStr;
use std::fmt::{Show, Formatter, Result};
use common::remove_trailing_crlf;
use std::result;

/// Contains an SMTP reply, with separed code and message
///
/// We do accept messages containing only a code, to comply with RFC5321
#[deriving(PartialEq,Eq,Clone)]
pub struct SmtpResponse {
    /// Server response code
    pub code: u16,
    /// Server response string
    pub message: Option<String>
}

impl Show for SmtpResponse {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write(
            match self.clone().message {
                Some(message) => format!("{} {}", self.code, message),
                None          => format!("{}", self.code)
            }.as_bytes()
        )
    }
}

// FromStr ?
impl FromStr for SmtpResponse {
    fn from_str(s: &str) -> Option<SmtpResponse> {
        // If the string is too short to be a response code
        if s.len() < 3 {
            None
        // If we have only a code, with or without a trailing space
        } else if s.len() == 3 || (s.len() == 4 && s.slice(3,4) == " ") {
            match from_str::<u16>(s.slice_to(3)) {
                Some(code) => Some(SmtpResponse{
                            code: code,
                            message: None
                        }),
                None         => None

            }
        // If we have a code and a message
        } else {
            match (
                from_str::<u16>(s.slice_to(3)),
                vec!(" ", "-").contains(&s.slice(3,4)),
                (remove_trailing_crlf(s.slice_from(4).to_string()))
            ) {
                (Some(code), true, message) => Some(SmtpResponse{
                            code: code,
                            message: Some(message)
                        }),
                _                           => None

            }
        }
    }
}

impl SmtpResponse {
    /// Checks the presence of the response code in the array of expected codes.
    pub fn with_code(&self, expected_codes: Vec<u16>) -> result::Result<SmtpResponse,SmtpResponse> {
        let response = self.clone();
        if expected_codes.contains(&self.code) {
            Ok(response)
        } else {
            Err(response)
        }
    }
}

#[cfg(test)]
mod test {
    use response::SmtpResponse;

    #[test]
    fn test_response_fmt() {
        assert_eq!(format!("{}", SmtpResponse{code: 200, message: Some("message".to_string())}), "200 message".to_string());
    }

    #[test]
    fn test_response_from_str() {
        assert_eq!(from_str::<SmtpResponse>("200 response message"),
            Some(SmtpResponse{
                code: 200,
                message: Some("response message".to_string())
            })
        );
        assert_eq!(from_str::<SmtpResponse>("200-response message"),
            Some(SmtpResponse{
                code: 200,
                message: Some("response message".to_string())
            })
        );
        assert_eq!(from_str::<SmtpResponse>("200"),
            Some(SmtpResponse{
                code: 200,
                message: None
            })
        );
        assert_eq!(from_str::<SmtpResponse>("200 "),
            Some(SmtpResponse{
                code: 200,
                message: None
            })
        );
        assert_eq!(from_str::<SmtpResponse>("200-response\r\nmessage"),
            Some(SmtpResponse{
                code: 200,
                message: Some("response\r\nmessage".to_string())
            })
        );
        assert_eq!(from_str::<SmtpResponse>("2000response message"), None);
        assert_eq!(from_str::<SmtpResponse>("20a response message"), None);
        assert_eq!(from_str::<SmtpResponse>("20 "), None);
        assert_eq!(from_str::<SmtpResponse>("20"), None);
        assert_eq!(from_str::<SmtpResponse>("2"), None);
        assert_eq!(from_str::<SmtpResponse>(""), None);
    }

    #[test]
    fn test_response_with_code() {
        assert_eq!(SmtpResponse{code: 200, message: Some("message".to_string())}.with_code(vec!(200)),
            Ok(SmtpResponse{code: 200, message: Some("message".to_string())}));
        assert_eq!(SmtpResponse{code: 400, message: Some("message".to_string())}.with_code(vec!(200)),
            Err(SmtpResponse{code: 400, message: Some("message".to_string())}));
        assert_eq!(SmtpResponse{code: 200, message: Some("message".to_string())}.with_code(vec!(200, 300)),
            Ok(SmtpResponse{code: 200, message: Some("message".to_string())}));
    }
}
