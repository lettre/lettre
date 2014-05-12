// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP commands and ESMTP features library

use std::fmt::{Show, Formatter, Result};
use std::result;
use std::io::net::ip::Port;
use std::from_str::FromStr;

use common::remove_trailing_crlf;

/// Default SMTP port
pub static SMTP_PORT: Port = 25;
//pub static SMTPS_PORT: Port = 465;
//pub static SUBMISSION_PORT: Port = 587;

/// Supported SMTP commands
///
/// We do not implement the following SMTP commands, as they were deprecated in RFC 5321
/// and must not be used by clients:
/// SEND, SOML, SAML, TURN
#[deriving(Eq,Clone)]
pub enum SmtpCommand<T> {
    /// Extended Hello command
    ExtendedHello(T),
    /// Hello command
    Hello(T),
    /// Mail command, takes optionnal options
    Mail(T, Option<Vec<T>>),
    /// Recipient command, takes optionnal options
    Recipient(T, Option<Vec<T>>),
    /// Data command
    Data,
    /// Reset command
    Reset,
    /// Verify command
    Verify(T),
    /// Expand command
    Expand(T),
    /// Help command, takes optionnal options
    Help(Option<T>),
    /// Noop command
    Noop,
    /// Quit command
    Quit,

}

impl<T: Show + Str> Show for SmtpCommand<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.buf.write(match *self {
            ExtendedHello(ref my_hostname) =>
                format!("EHLO {}", my_hostname.clone()),
            Hello(ref my_hostname) =>
                format!("HELO {}", my_hostname.clone()),
            Mail(ref from_address, None) =>
                format!("MAIL FROM:<{}>", from_address.clone()),
            Mail(ref from_address, Some(ref options)) =>
                format!("MAIL FROM:<{}> {}", from_address.clone(), options.connect(" ")),
            Recipient(ref to_address, None) =>
                format!("RCPT TO:<{}>", to_address.clone()),
            Recipient(ref to_address, Some(ref options)) =>
                format!("RCPT TO:<{}> {}", to_address.clone(), options.connect(" ")),
            Data => "DATA".to_owned(),
            Reset => "RSET".to_owned(),
            Verify(ref address) =>
                format!("VRFY {}", address.clone()),
            Expand(ref address) =>
                format!("EXPN {}", address.clone()),
            Help(None) => "HELP".to_owned(),
            Help(Some(ref argument)) =>
                format!("HELP {}", argument.clone()),
            Noop => "NOOP".to_owned(),
            Quit => "QUIT".to_owned(),
        }.as_bytes())
    }
}

/// Supported ESMTP keywords
#[deriving(Eq,Clone)]
pub enum EsmtpParameter {
    /// 8BITMIME keyword
    ///
    /// RFC 6152 : https://tools.ietf.org/html/rfc6152
    EightBitMime,
    /// SIZE keyword
    ///
    /// RFC 1427 : https://tools.ietf.org/html/rfc1427
    Size(uint)
}

impl Show for EsmtpParameter {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.buf.write(
            match self {
                &EightBitMime   => "8BITMIME".to_owned(),
                &Size(ref size) => format!("SIZE={}", size)
            }.as_bytes()
        )
    }
}

impl FromStr for EsmtpParameter {
    fn from_str(s: &str) -> Option<EsmtpParameter> {
        let splitted : Vec<&str> = s.splitn(' ', 1).collect();
        match splitted.len() {
            1 => match *splitted.get(0) {
                     "8BITMIME" => Some(EightBitMime),
                     _          => None
                 },
            2 => match (*splitted.get(0), from_str::<uint>(*splitted.get(1))) {
                     ("SIZE", Some(size)) => Some(Size(size)),
                     _                    => None
                 },
            _          => None
        }
    }
}

impl EsmtpParameter {
    /// Checks if the ESMTP keyword is the same
    pub fn same_keyword_as(&self, other: EsmtpParameter) -> bool {
        if *self == other {
            return true;
        }
        match (*self, other) {
            (Size(_), Size(_)) => true,
            _                  => false
        }
    }
}

/// Contains an SMTP reply, with separed code and message
///
/// We do accept messages containing only a code, to comply with RFC5321
#[deriving(Clone, Eq)]
pub struct SmtpResponse<T> {
    /// Server response code
    pub code: u16,
    /// Server response string
    pub message: Option<T>
}

impl<T: Show + Clone> Show for SmtpResponse<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.buf.write(
            match self.clone().message {
                Some(message) => format!("{} {}", self.code.to_str(), message),
                None          => self.code.to_str()
            }.as_bytes()
        )
    }
}

// FromStr ?
impl FromStr for SmtpResponse<StrBuf> {
    fn from_str(s: &str) -> Option<SmtpResponse<StrBuf>> {
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
                StrBuf::from_str(remove_trailing_crlf(s.slice_from(4).to_owned()))
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

impl<T: Clone> SmtpResponse<T> {
    /// Checks the presence of the response code in the array of expected codes.
    pub fn with_code(&self, expected_codes: Vec<u16>) -> result::Result<SmtpResponse<T>,SmtpResponse<T>> {
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
    use super::{SmtpCommand, EsmtpParameter, SmtpResponse};

    #[test]
    fn test_command_fmt() {
        let noop: SmtpCommand<StrBuf> = super::Noop;
        assert_eq!(format!("{}", noop), "NOOP".to_owned());
        assert_eq!(format!("{}", super::ExtendedHello("me")), "EHLO me".to_owned());
        assert_eq!(format!("{}",
            super::Mail("test", Some(vec!("option")))), "MAIL FROM:<test> option".to_owned()
        );
    }

    #[test]
    fn test_esmtp_parameter_same_keyword_as() {
        assert_eq!(super::EightBitMime.same_keyword_as(super::EightBitMime), true);
        assert_eq!(super::Size(42).same_keyword_as(super::Size(42)), true);
        assert_eq!(super::Size(42).same_keyword_as(super::Size(43)), true);
        assert_eq!(super::Size(42).same_keyword_as(super::EightBitMime), false);
    }

    #[test]
    fn test_esmtp_parameter_fmt() {
        assert_eq!(format!("{}", super::EightBitMime), "8BITMIME".to_owned());
        assert_eq!(format!("{}", super::Size(42)), "SIZE=42".to_owned());
    }

    #[test]
    fn test_esmtp_parameter_from_str() {
        assert_eq!(from_str::<EsmtpParameter>("8BITMIME"), Some(super::EightBitMime));
        assert_eq!(from_str::<EsmtpParameter>("SIZE 42"), Some(super::Size(42)));
        assert_eq!(from_str::<EsmtpParameter>("SIZ 42"), None);
        assert_eq!(from_str::<EsmtpParameter>("SIZE 4a2"), None);
        // TODO: accept trailing spaces ?
        assert_eq!(from_str::<EsmtpParameter>("SIZE 42 "), None);
    }

    #[test]
    fn test_smtp_response_fmt() {
        assert_eq!(format!("{}", SmtpResponse{code: 200, message: Some("message")}), "200 message".to_owned());
    }

    #[test]
    fn test_smtp_response_from_str() {
        assert_eq!(from_str::<SmtpResponse<StrBuf>>("200 response message"),
            Some(SmtpResponse{
                code: 200,
                message: Some(StrBuf::from_str("response message"))
            })
        );
        assert_eq!(from_str::<SmtpResponse<StrBuf>>("200-response message"),
            Some(SmtpResponse{
                code: 200,
                message: Some(StrBuf::from_str("response message"))
            })
        );
        assert_eq!(from_str::<SmtpResponse<StrBuf>>("200"),
            Some(SmtpResponse{
                code: 200,
                message: None
            })
        );
        assert_eq!(from_str::<SmtpResponse<StrBuf>>("200 "),
            Some(SmtpResponse{
                code: 200,
                message: None
            })
        );
        assert_eq!(from_str::<SmtpResponse<StrBuf>>("200-response\r\nmessage"),
            Some(SmtpResponse{
                code: 200,
                message: Some(StrBuf::from_str("response\r\nmessage"))
            })
        );
        assert_eq!(from_str::<SmtpResponse<StrBuf>>("2000response message"), None);
        assert_eq!(from_str::<SmtpResponse<StrBuf>>("20a response message"), None);
        assert_eq!(from_str::<SmtpResponse<StrBuf>>("20 "), None);
        assert_eq!(from_str::<SmtpResponse<StrBuf>>("20"), None);
        assert_eq!(from_str::<SmtpResponse<StrBuf>>("2"), None);
        assert_eq!(from_str::<SmtpResponse<StrBuf>>(""), None);
    }

    #[test]
    fn test_smtp_response_with_code() {
        assert_eq!(SmtpResponse{code: 200, message: Some("message")}.with_code(vec!(200)),
            Ok(SmtpResponse{code: 200, message: Some("message")}));
        assert_eq!(SmtpResponse{code: 400, message: Some("message")}.with_code(vec!(200)),
            Err(SmtpResponse{code: 400, message: Some("message")}));
        assert_eq!(SmtpResponse{code: 200, message: Some("message")}.with_code(vec!(200, 300)),
            Ok(SmtpResponse{code: 200, message: Some("message")}));
    }
}
