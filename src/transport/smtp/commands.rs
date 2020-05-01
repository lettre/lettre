//! SMTP commands

use crate::{
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        error::Error,
        extension::{ClientId, MailParameter, RcptParameter},
        response::Response,
    },
    Address,
};
use log::debug;
use std::{
    convert::AsRef,
    fmt::{self, Display, Formatter},
};

/// EHLO command
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ehlo {
    client_id: ClientId,
}

impl Display for Ehlo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        #[allow(clippy::write_with_newline)]
        write!(f, "EHLO {}\r\n", self.client_id)
    }
}

impl Ehlo {
    /// Creates a EHLO command
    pub fn new(client_id: ClientId) -> Ehlo {
        Ehlo { client_id }
    }
}

/// STARTTLS command
#[derive(PartialEq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Starttls;

impl Display for Starttls {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("STARTTLS\r\n")
    }
}

/// MAIL command
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mail {
    sender: Option<Address>,
    parameters: Vec<MailParameter>,
}

impl Display for Mail {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "MAIL FROM:<{}>",
            self.sender.as_ref().map(|s| s.as_ref()).unwrap_or("")
        )?;
        for parameter in &self.parameters {
            write!(f, " {}", parameter)?;
        }
        f.write_str("\r\n")
    }
}

impl Mail {
    /// Creates a MAIL command
    pub fn new(sender: Option<Address>, parameters: Vec<MailParameter>) -> Mail {
        Mail { sender, parameters }
    }
}

/// RCPT command
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rcpt {
    recipient: Address,
    parameters: Vec<RcptParameter>,
}

impl Display for Rcpt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "RCPT TO:<{}>", self.recipient)?;
        for parameter in &self.parameters {
            write!(f, " {}", parameter)?;
        }
        f.write_str("\r\n")
    }
}

impl Rcpt {
    /// Creates an RCPT command
    pub fn new(recipient: Address, parameters: Vec<RcptParameter>) -> Rcpt {
        Rcpt {
            recipient,
            parameters,
        }
    }
}

/// DATA command
#[derive(PartialEq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Data;

impl Display for Data {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("DATA\r\n")
    }
}

/// QUIT command
#[derive(PartialEq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Quit;

impl Display for Quit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("QUIT\r\n")
    }
}

/// NOOP command
#[derive(PartialEq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Noop;

impl Display for Noop {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("NOOP\r\n")
    }
}

/// HELP command
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Help {
    argument: Option<String>,
}

impl Display for Help {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("HELP")?;
        if self.argument.is_some() {
            write!(f, " {}", self.argument.as_ref().unwrap())?;
        }
        f.write_str("\r\n")
    }
}

impl Help {
    /// Creates an HELP command
    pub fn new(argument: Option<String>) -> Help {
        Help { argument }
    }
}

/// VRFY command
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vrfy {
    argument: String,
}

impl Display for Vrfy {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        #[allow(clippy::write_with_newline)]
        write!(f, "VRFY {}\r\n", self.argument)
    }
}

impl Vrfy {
    /// Creates a VRFY command
    pub fn new(argument: String) -> Vrfy {
        Vrfy { argument }
    }
}

/// EXPN command
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Expn {
    argument: String,
}

impl Display for Expn {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        #[allow(clippy::write_with_newline)]
        write!(f, "EXPN {}\r\n", self.argument)
    }
}

impl Expn {
    /// Creates an EXPN command
    pub fn new(argument: String) -> Expn {
        Expn { argument }
    }
}

/// RSET command
#[derive(PartialEq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rset;

impl Display for Rset {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("RSET\r\n")
    }
}

/// AUTH command
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Auth {
    mechanism: Mechanism,
    credentials: Credentials,
    challenge: Option<String>,
    response: Option<String>,
}

impl Display for Auth {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let encoded_response = self
            .response
            .as_ref()
            .map(|r| base64::encode_config(r.as_bytes(), base64::STANDARD));

        if self.mechanism.supports_initial_response() {
            write!(f, "AUTH {} {}", self.mechanism, encoded_response.unwrap())?;
        } else {
            match encoded_response {
                Some(response) => f.write_str(&response)?,
                None => write!(f, "AUTH {}", self.mechanism)?,
            }
        }
        f.write_str("\r\n")
    }
}

impl Auth {
    /// Creates an AUTH command (from a challenge if provided)
    pub fn new(
        mechanism: Mechanism,
        credentials: Credentials,
        challenge: Option<String>,
    ) -> Result<Auth, Error> {
        let response = if mechanism.supports_initial_response() || challenge.is_some() {
            Some(mechanism.response(&credentials, challenge.as_deref())?)
        } else {
            None
        };
        Ok(Auth {
            mechanism,
            credentials,
            challenge,
            response,
        })
    }

    /// Creates an AUTH command from a response that needs to be a
    /// valid challenge (with 334 response code)
    pub fn new_from_response(
        mechanism: Mechanism,
        credentials: Credentials,
        response: &Response,
    ) -> Result<Auth, Error> {
        if !response.has_code(334) {
            return Err(Error::ResponseParsing("Expecting a challenge"));
        }

        let encoded_challenge = response
            .first_word()
            .ok_or(Error::ResponseParsing("Could not read auth challenge"))?;
        debug!("auth encoded challenge: {}", encoded_challenge);

        let decoded_challenge = String::from_utf8(base64::decode(&encoded_challenge)?)?;
        debug!("auth decoded challenge: {}", decoded_challenge);

        let response = Some(mechanism.response(&credentials, Some(decoded_challenge.as_ref()))?);

        Ok(Auth {
            mechanism,
            credentials,
            challenge: Some(decoded_challenge),
            response,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::transport::smtp::extension::MailBodyParameter;
    use std::str::FromStr;

    #[test]
    fn test_display() {
        let id = ClientId::Domain("localhost".to_string());
        let email = Address::from_str("test@example.com").unwrap();
        let mail_parameter = MailParameter::Other {
            keyword: "TEST".to_string(),
            value: Some("value".to_string()),
        };
        let rcpt_parameter = RcptParameter::Other {
            keyword: "TEST".to_string(),
            value: Some("value".to_string()),
        };
        assert_eq!(format!("{}", Ehlo::new(id)), "EHLO localhost\r\n");
        assert_eq!(
            format!("{}", Mail::new(Some(email.clone()), vec![])),
            "MAIL FROM:<test@example.com>\r\n"
        );
        assert_eq!(format!("{}", Mail::new(None, vec![])), "MAIL FROM:<>\r\n");
        assert_eq!(
            format!(
                "{}",
                Mail::new(Some(email.clone()), vec![MailParameter::Size(42)])
            ),
            "MAIL FROM:<test@example.com> SIZE=42\r\n"
        );
        assert_eq!(
            format!(
                "{}",
                Mail::new(
                    Some(email.clone()),
                    vec![
                        MailParameter::Size(42),
                        MailParameter::Body(MailBodyParameter::EightBitMime),
                        mail_parameter,
                    ],
                )
            ),
            "MAIL FROM:<test@example.com> SIZE=42 BODY=8BITMIME TEST=value\r\n"
        );
        assert_eq!(
            format!("{}", Rcpt::new(email.clone(), vec![])),
            "RCPT TO:<test@example.com>\r\n"
        );
        assert_eq!(
            format!("{}", Rcpt::new(email.clone(), vec![rcpt_parameter])),
            "RCPT TO:<test@example.com> TEST=value\r\n"
        );
        assert_eq!(format!("{}", Quit), "QUIT\r\n");
        assert_eq!(format!("{}", Data), "DATA\r\n");
        assert_eq!(format!("{}", Noop), "NOOP\r\n");
        assert_eq!(format!("{}", Help::new(None)), "HELP\r\n");
        assert_eq!(
            format!("{}", Help::new(Some("test".to_string()))),
            "HELP test\r\n"
        );
        assert_eq!(
            format!("{}", Vrfy::new("test".to_string())),
            "VRFY test\r\n"
        );
        assert_eq!(
            format!("{}", Expn::new("test".to_string())),
            "EXPN test\r\n"
        );
        assert_eq!(format!("{}", Rset), "RSET\r\n");
        let credentials = Credentials::new("user".to_string(), "password".to_string());
        assert_eq!(
            format!(
                "{}",
                Auth::new(Mechanism::Plain, credentials.clone(), None).unwrap()
            ),
            "AUTH PLAIN AHVzZXIAcGFzc3dvcmQ=\r\n"
        );
        assert_eq!(
            format!(
                "{}",
                Auth::new(Mechanism::Login, credentials.clone(), None).unwrap()
            ),
            "AUTH LOGIN\r\n"
        );
    }
}
