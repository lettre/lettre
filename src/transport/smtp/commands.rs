//! SMTP commands

use std::fmt::{self, Display, Formatter};

use rsasl::prelude::Session;

use crate::{
    address::Address,
    transport::smtp::{
        error::{self, Error, Kind},
        extension::{ClientId, MailParameter, RcptParameter},
        response::Response,
    },
};

/// EHLO command
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ehlo {
    client_id: ClientId,
}

impl Display for Ehlo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
#[derive(PartialEq, Eq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Starttls;

impl Display for Starttls {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("STARTTLS\r\n")
    }
}

/// MAIL command
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mail {
    sender: Option<Address>,
    parameters: Vec<MailParameter>,
}

impl Display for Mail {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MAIL FROM:<{}>",
            self.sender.as_ref().map_or("", |s| s.as_ref())
        )?;
        for parameter in &self.parameters {
            write!(f, " {parameter}")?;
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
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rcpt {
    recipient: Address,
    parameters: Vec<RcptParameter>,
}

impl Display for Rcpt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "RCPT TO:<{}>", self.recipient)?;
        for parameter in &self.parameters {
            write!(f, " {parameter}")?;
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
#[derive(PartialEq, Eq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Data;

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("DATA\r\n")
    }
}

/// QUIT command
#[derive(PartialEq, Eq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Quit;

impl Display for Quit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("QUIT\r\n")
    }
}

/// NOOP command
#[derive(PartialEq, Eq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Noop;

impl Display for Noop {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("NOOP\r\n")
    }
}

/// HELP command
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Help {
    argument: Option<String>,
}

impl Display for Help {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("HELP")?;
        if let Some(argument) = &self.argument {
            write!(f, " {argument}")?;
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
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vrfy {
    argument: String,
}

impl Display for Vrfy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Expn {
    argument: String,
}

impl Display for Expn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
#[derive(PartialEq, Eq, Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rset;

impl Display for Rset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("RSET\r\n")
    }
}

/// AUTH command
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Auth {
    msg: AuthMsg,
}
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum AuthMsg {
    Initial(String, Option<String>),
    Contd(String),
}

impl Display for Auth {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.msg {
            AuthMsg::Initial(mechname, Some(response)) => {
                write!(f, "AUTH {mechname} {}", response.as_str())?;
            }
            AuthMsg::Initial(mechname, None) => {
                write!(f, "AUTH {mechname}")?;
            }
            AuthMsg::Contd(response) => {
                write!(f, "AUTH {}", response.as_str())?;
            }
        }
        f.write_str("\r\n")
    }
}

impl Auth {
    /// Creates an AUTH command (from a challenge if provided)
    pub fn initial(session: &mut Session) -> Result<Self, Error> {
        let name = session.get_mechname().as_str().to_string();
        let response = if session.are_we_first() {
            let mut out = Vec::new();
            session
                .step64(None, &mut out)
                .map_err(|error| Error::new(Kind::Client, Some(Box::new(error))))?;
            Some(String::from_utf8(out).expect("base64 encoded output is not UTF-8"))
        } else {
            None
        };
        Ok(Self {
            msg: AuthMsg::Initial(name, response),
        })
    }

    pub fn from_response(session: &mut Session, response: &Response) -> Result<Self, Error> {
        if !response.has_code(334) {
            return Err(error::response("Expecting a challenge"));
        }

        let encoded_challenge = response
            .first_word()
            .ok_or_else(|| error::response("Could not read auth challenge"))?;
        #[cfg(feature = "tracing")]
        tracing::debug!("auth encoded challenge: {}", encoded_challenge);

        let mut out = Vec::new();
        session
            .step64(Some(encoded_challenge.as_bytes()), &mut out)
            .map_err(|error| Error::new(Kind::Client, Some(Box::new(error))))?;
        let output = String::from_utf8(out).expect("base64 encoded output is not UTF-8");

        Ok(Self {
            msg: AuthMsg::Contd(output),
        })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use crate::transport::smtp::extension::MailBodyParameter;

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
            format!("{}", Rcpt::new(email, vec![rcpt_parameter])),
            "RCPT TO:<test@example.com> TEST=value\r\n"
        );
        assert_eq!(format!("{Quit}"), "QUIT\r\n");
        assert_eq!(format!("{Data}"), "DATA\r\n");
        assert_eq!(format!("{Noop}"), "NOOP\r\n");
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
        assert_eq!(format!("{Rset}"), "RSET\r\n");
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
                Auth::new(Mechanism::Login, credentials, None).unwrap()
            ),
            "AUTH LOGIN\r\n"
        );
    }
}
