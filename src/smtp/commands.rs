/*!
 * SMTP commands library
 *
 * RFC 5321 : http://tools.ietf.org/html/rfc5321#section-4.1
 */

use std::fmt;
use std::io;

/*
 * HELO <SP> <domain> <CRLF>
 * MAIL <SP> FROM:<reverse-path> <CRLF>
 * RCPT <SP> TO:<forward-path> <CRLF>
 * DATA <CRLF>
 * RSET <CRLF>
 * SEND <SP> FROM:<reverse-path> <CRLF>
 * SOML <SP> FROM:<reverse-path> <CRLF>
 * SAML <SP> FROM:<reverse-path> <CRLF>
 * VRFY <SP> <string> <CRLF>
 * EXPN <SP> <string> <CRLF>
 * HELP [<SP> <string>] <CRLF>
 * NOOP <CRLF>
 * QUIT <CRLF>
 * TURN <CRLF>
 */

/// List of SMTP commands
#[deriving(Eq,Clone)]
pub enum Command {
    /// Hello command
    HELO,
    /// Extended Hello command
    EHLO,
    /// Mail command
    MAIL,
    /// Recipient command
    RCPT,
    /// Data command
    DATA,
    /// Reset command
    RSET,
    /// Send command, deprecated in RFC 5321
    SEND,
    /// Send Or Mail command, deprecated in RFC 5321
    SOML,
    /// Send And Mail command, deprecated in RFC 5321
    SAML,
    /// Verify command
    VRFY,
    /// Expand command
    EXPN,
    /// Help command
    HELP,
    /// Noop command
    NOOP,
    /// Quit command
    QUIT,
    /// Turn command, deprecated in RFC 5321
    TURN,
}

impl Command {
    /// Tell if the command accetps an string argument.
    pub fn takes_argument(&self) -> bool{
        match *self {
            EHLO => true,
            HELO => true,
            MAIL => true,
            RCPT => true,
            DATA => false,
            RSET => false,
            SEND => true,
            SOML => true,
            SAML => true,
            VRFY => true,
            EXPN => true,
            HELP => true,
            NOOP => false,
            QUIT => false,
            TURN => false,
        }
    }

    /// Tell if an argument is needed by the command.
    pub fn needs_argument(&self) -> bool {
        match *self {
            EHLO => true,
            HELO => true,
            MAIL => true,
            RCPT => true,
            DATA => false,
            RSET => false,
            SEND => true,
            SOML => true,
            SAML => true,
            VRFY => true,
            EXPN => true,
            HELP => false,
            NOOP => false,
            QUIT => false,
            TURN => false,
        }
    }
}

impl fmt::Show for Command {
    /// Format SMTP command display
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), io::IoError> {
        f.buf.write(match *self {
            EHLO => "EHLO",
            HELO => "HELO",
            MAIL => "MAIL FROM:",
            RCPT => "RCPT TO:",
            DATA => "DATA",
            RSET => "RSET",
            SEND => "SEND TO:",
            SOML => "SOML TO:",
            SAML => "SAML TO:",
            VRFY => "VRFY",
            EXPN => "EXPN",
            HELP => "HELP",
            NOOP => "NOOP",
            QUIT => "QUIT",
            TURN => "TURN"
        }.as_bytes())
    }
}

/// Structure for a complete SMTP command, containing an optionnal string argument.
pub struct SmtpCommand {
    /// The SMTP command (e.g. MAIL, QUIT, ...)
    command: Command,
    /// An optionnal argument to the command
    argument: Option<~str>
}

impl SmtpCommand {
    /// Return a new structure from the name of the command and an optionnal argument.
    pub fn new(command: Command, argument: Option<~str>) -> SmtpCommand {
        match (command.takes_argument(), command.needs_argument(), argument.clone()) {
            (true, true, None)      => fail!("Wrong SMTP syntax : argument needed"),
            (false, false, Some(x)) => fail!("Wrong SMTP syntax : {:s} not accepted", x),
            _                       => SmtpCommand {command: command, argument: argument}
        }
    }
}

impl fmt::Show for SmtpCommand {
    /// Return the formatted command, ready to be used in an SMTP session.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), io::IoError> {
        f.buf.write(
            match (self.command.takes_argument(), self.command.needs_argument(), self.argument.clone()) {
                (true, _, Some(argument)) => format!("{} {}", self.command, argument),
                (_, false, None)   => format!("{}", self.command),
                _                  => fail!("Wrong SMTP syntax")
            }.as_bytes()
        )
    }
}

/// Supported ESMTP keywords
#[deriving(Eq,Clone)]
pub enum EhloKeyword {
    /// STARTTLS keyword
    STARTTLS,
    /// 8BITMIME keyword
    BITMIME,
    /// SMTP authentification
    AUTH
}


#[cfg(test)]
mod test {
    use super::{Command, SmtpCommand};

    #[test]
    fn test_command_parameters() {
        assert!((super::HELP).takes_argument() == true);
        assert!((super::RSET).takes_argument() == false);
        assert!((super::HELO).needs_argument() == true);
    }

    #[test]
    fn test_to_str() {
        assert!(super::TURN.to_str() == ~"TURN");
    }

    #[test]
    fn test_fmt() {
        assert!(format!("{}", super::TURN) == ~"TURN");
    }

    #[test]
    fn test_get_simple_command() {
        assert!(SmtpCommand::new(super::TURN, None).to_str() == ~"TURN");
    }

    #[test]
    fn test_get_argument_command() {
        assert!(SmtpCommand::new(super::EHLO, Some(~"example.example")).to_str() == ~"EHLO example.example");
    }
}
