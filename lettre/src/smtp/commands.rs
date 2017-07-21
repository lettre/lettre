//! SMTP commands

use EmailAddress;
use base64;
use smtp::CRLF;
use smtp::authentication::{Credentials, Mechanism};
use smtp::error::Error;
use smtp::extension::{MailParameter, RcptParameter};
use smtp::extension::ClientId;
use smtp::response::Response;
use std::fmt;
use std::fmt::{Display, Formatter};

/// EHLO command
#[derive(PartialEq, Clone, Debug)]
pub struct EhloCommand {
    client_id: ClientId,
}

impl Display for EhloCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "EHLO {}", self.client_id)?;
        f.write_str(CRLF)
    }
}

impl EhloCommand {
    /// Creates a EHLO command
    pub fn new(client_id: ClientId) -> EhloCommand {
        EhloCommand { client_id: client_id }
    }
}

/// STARTTLS command
#[derive(PartialEq, Clone, Debug)]
pub struct StarttlsCommand;

impl Display for StarttlsCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("STARTTLS")?;
        f.write_str(CRLF)
    }
}

/// MAIL command
#[derive(PartialEq, Clone, Debug)]
pub struct MailCommand {
    sender: Option<EmailAddress>,
    parameters: Vec<MailParameter>,
}

impl Display for MailCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "MAIL FROM:<{}>",
            match self.sender {
                Some(ref address) => address.to_string(),
                None => "".to_string(),
            }
        )?;
        for parameter in &self.parameters {
            write!(f, " {}", parameter)?;
        }
        f.write_str(CRLF)
    }
}

impl MailCommand {
    /// Creates a MAIL command
    pub fn new(sender: Option<EmailAddress>, parameters: Vec<MailParameter>) -> MailCommand {
        MailCommand {
            sender: sender,
            parameters: parameters,
        }
    }
}

/// RCPT command
#[derive(PartialEq, Clone, Debug)]
pub struct RcptCommand {
    recipient: EmailAddress,
    parameters: Vec<RcptParameter>,
}

impl Display for RcptCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "RCPT TO:<{}>", self.recipient)?;
        for parameter in &self.parameters {
            write!(f, " {}", parameter)?;
        }
        f.write_str(CRLF)
    }
}

impl RcptCommand {
    /// Creates an RCPT command
    pub fn new(recipient: EmailAddress, parameters: Vec<RcptParameter>) -> RcptCommand {
        RcptCommand {
            recipient: recipient,
            parameters: parameters,
        }
    }
}

/// DATA command
#[derive(PartialEq, Clone, Debug)]
pub struct DataCommand;

impl Display for DataCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("DATA")?;
        f.write_str(CRLF)
    }
}

/// QUIT command
#[derive(PartialEq, Clone, Debug)]
pub struct QuitCommand;

impl Display for QuitCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("QUIT")?;
        f.write_str(CRLF)
    }
}

/// NOOP command
#[derive(PartialEq, Clone, Debug)]
pub struct NoopCommand;

impl Display for NoopCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("NOOP")?;
        f.write_str(CRLF)
    }
}

/// HELP command
#[derive(PartialEq, Clone, Debug)]
pub struct HelpCommand {
    argument: Option<String>,
}

impl Display for HelpCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("HELP")?;
        if self.argument.is_some() {
            write!(f, " {}", self.argument.as_ref().unwrap())?;
        }
        f.write_str(CRLF)
    }
}

impl HelpCommand {
    /// Creates an HELP command
    pub fn new(argument: Option<String>) -> HelpCommand {
        HelpCommand { argument: argument }
    }
}

/// VRFY command
#[derive(PartialEq, Clone, Debug)]
pub struct VrfyCommand {
    argument: String,
}

impl Display for VrfyCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "VRFY {}", self.argument)?;
        f.write_str(CRLF)
    }
}

impl VrfyCommand {
    /// Creates a VRFY command
    pub fn new(argument: String) -> VrfyCommand {
        VrfyCommand { argument: argument }
    }
}

/// EXPN command
#[derive(PartialEq, Clone, Debug)]
pub struct ExpnCommand {
    argument: String,
}

impl Display for ExpnCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "EXPN {}", self.argument)?;
        f.write_str(CRLF)
    }
}

impl ExpnCommand {
    /// Creates an EXPN command
    pub fn new(argument: String) -> ExpnCommand {
        ExpnCommand { argument: argument }
    }
}

/// RSET command
#[derive(PartialEq, Clone, Debug)]
pub struct RsetCommand;

impl Display for RsetCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("RSET")?;
        f.write_str(CRLF)
    }
}

/// AUTH command
#[derive(PartialEq, Clone, Debug)]
pub struct AuthCommand {
    mechanism: Mechanism,
    credentials: Credentials,
    challenge: Option<String>,
    response: Option<String>,
}

impl Display for AuthCommand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let encoded_response = if self.response.is_some() {
            Some(base64::encode_config(
                self.response.as_ref().unwrap().as_bytes(),
                base64::STANDARD,
            ))
        } else {
            None
        };

        if self.mechanism.supports_initial_response() {
            write!(f, "AUTH {} {}", self.mechanism, encoded_response.unwrap(),)?;
        } else {
            match encoded_response {
                Some(response) => f.write_str(&response)?,
                None => write!(f, "AUTH {}", self.mechanism)?,
            }
        }
        f.write_str(CRLF)
    }
}

impl AuthCommand {
    /// Creates an AUTH command (from a challenge if provided)
    pub fn new(
        mechanism: Mechanism,
        credentials: Credentials,
        challenge: Option<String>,
    ) -> Result<AuthCommand, Error> {
        let response = if mechanism.supports_initial_response() || challenge.is_some() {
            Some(mechanism.response(
                &credentials,
                challenge.as_ref().map(String::as_str),
            )?)
        } else {
            None
        };
        Ok(AuthCommand {
            mechanism: mechanism,
            credentials: credentials,
            challenge: challenge,
            response: response,
        })
    }

    /// Creates an AUTH command from a response that needs to be a
    /// valid challenge (with 334 response code)
    pub fn new_from_response(
        mechanism: Mechanism,
        credentials: Credentials,
        response: Response,
    ) -> Result<AuthCommand, Error> {
        if !response.has_code(334) {
            return Err(Error::ResponseParsing("Expecting a challenge"));
        }

        let encoded_challenge = match response.first_word() {
            Some(challenge) => challenge.to_string(),
            None => return Err(Error::ResponseParsing("Could not read auth challenge")),
        };

        debug!("auth encoded challenge: {}", encoded_challenge);

        let decoded_challenge = match base64::decode(&encoded_challenge) {
            Ok(challenge) => {
                match String::from_utf8(challenge) {
                    Ok(value) => value,
                    Err(error) => return Err(Error::Utf8Parsing(error)),
                }
            }
            Err(error) => return Err(Error::ChallengeParsing(error)),
        };

        debug!("auth decoded challenge: {}", decoded_challenge);

        let response = Some(mechanism.response(
            &credentials,
            Some(decoded_challenge.as_ref()),
        )?);

        Ok(AuthCommand {
            mechanism: mechanism,
            credentials: credentials,
            challenge: Some(decoded_challenge),
            response: response,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use smtp::extension::MailBodyParameter;
    #[cfg(feature = "crammd5-auth")]
    use smtp::response::Code;
    #[cfg(feature = "crammd5-auth")]
    use std::str::FromStr;

    #[test]
    fn test_display() {
        let id = ClientId::Domain("localhost".to_string());
        let email = EmailAddress::new("test@example.com".to_string());
        let mail_parameter = MailParameter::Other {
            keyword: "TEST".to_string(),
            value: Some("value".to_string()),
        };
        let rcpt_parameter = RcptParameter::Other {
            keyword: "TEST".to_string(),
            value: Some("value".to_string()),
        };
        assert_eq!(format!("{}", EhloCommand::new(id)), "EHLO localhost\r\n");
        assert_eq!(
            format!("{}", MailCommand::new(Some(email.clone()), vec![])),
            "MAIL FROM:<test@example.com>\r\n"
        );
        assert_eq!(
            format!("{}", MailCommand::new(None, vec![])),
            "MAIL FROM:<>\r\n"
        );
        assert_eq!(
            format!(
                "{}",
                MailCommand::new(Some(email.clone()), vec![MailParameter::Size(42)])
            ),
            "MAIL FROM:<test@example.com> SIZE=42\r\n"
        );
        assert_eq!(
            format!(
                "{}",
                MailCommand::new(
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
            format!("{}", RcptCommand::new(email.clone(), vec![])),
            "RCPT TO:<test@example.com>\r\n"
        );
        assert_eq!(
            format!("{}", RcptCommand::new(email.clone(), vec![rcpt_parameter])),
            "RCPT TO:<test@example.com> TEST=value\r\n"
        );
        assert_eq!(format!("{}", QuitCommand), "QUIT\r\n");
        assert_eq!(format!("{}", DataCommand), "DATA\r\n");
        assert_eq!(format!("{}", NoopCommand), "NOOP\r\n");
        assert_eq!(format!("{}", HelpCommand::new(None)), "HELP\r\n");
        assert_eq!(
            format!("{}", HelpCommand::new(Some("test".to_string()))),
            "HELP test\r\n"
        );
        assert_eq!(
            format!("{}", VrfyCommand::new("test".to_string())),
            "VRFY test\r\n"
        );
        assert_eq!(
            format!("{}", ExpnCommand::new("test".to_string())),
            "EXPN test\r\n"
        );
        assert_eq!(format!("{}", RsetCommand), "RSET\r\n");
        let credentials = Credentials::new("user".to_string(), "password".to_string());
        assert_eq!(
            format!(
                "{}",
                AuthCommand::new(Mechanism::Plain, credentials.clone(), None).unwrap()
            ),
            "AUTH PLAIN AHVzZXIAcGFzc3dvcmQ=\r\n"
        );
        #[cfg(feature = "crammd5-auth")]
        assert_eq!(
            format!(
                "{}",
                AuthCommand::new(
                    Mechanism::CramMd5,
                    credentials.clone(),
                    Some("test".to_string()),
                ).unwrap()
            ),
            "dXNlciAzMTYxY2NmZDdmMjNlMzJiYmMzZTQ4NjdmYzk0YjE4Nw==\r\n"
        );
        assert_eq!(
            format!(
                "{}",
                AuthCommand::new(Mechanism::Login, credentials.clone(), None).unwrap()
            ),
            "AUTH LOGIN\r\n"
        );
        #[cfg(feature = "crammd5-auth")]
        assert_eq!(
            format!(
                "{}",
                AuthCommand::new_from_response(
                    Mechanism::CramMd5,
                    credentials.clone(),
                    Response::new(Code::from_str("334").unwrap(), vec!["dGVzdAo=".to_string()]),
                ).unwrap()
            ),
            "dXNlciA1NTIzNThiMzExOWFjOWNkYzM2YWRiN2MxNWRmMWJkNw==\r\n"
        );
    }
}
