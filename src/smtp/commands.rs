/*!
 * SMTP commands and ESMTP features library
 *
 * RFC 5321 : https://tools.ietf.org/html/rfc5321#section-4.1
 */

use std::fmt;
use std::io;
use std::from_str;
use std::io::IoError;

/// List of SMTP commands
#[deriving(Eq,Clone)]
pub enum Command {
    /// Extended Hello command
    Ehlo,
    /// Hello command
    Helo,
    /// Mail command
    Mail,
    /// Recipient command
    Rcpt,
    /// Data command
    Data,
    /// Reset command
    Rset,
    /// Send command, deprecated in RFC 5321
    Send,
    /// Send Or Mail command, deprecated in RFC 5321
    Soml,
    /// Send And Mail command, deprecated in RFC 5321
    Saml,
    /// Verify command
    Vrfy,
    /// Expand command
    Expn,
    /// Help command
    Help,
    /// Noop command
    Noop,
    /// Quit command
    Quit,
    /// Turn command, deprecated in RFC 5321
    Turn,
}

impl Command {
    /// Tell if the command accetps an string argument.
    pub fn takes_argument(&self) -> bool{
        match *self {
            Ehlo => true,
            Helo => true,
            Mail => true,
            Rcpt => true,
            Data => false,
            Rset => false,
            Send => true,
            Soml => true,
            Saml => true,
            Vrfy => true,
            Expn => true,
            Help => true,
            Noop => false,
            Quit => false,
            Turn => false,
        }
    }

    /// Tell if an argument is needed by the command.
    pub fn needs_argument(&self) -> bool {
        match *self {
            Ehlo => true,
            Helo => true,
            Mail => true,
            Rcpt => true,
            Data => false,
            Rset => false,
            Send => true,
            Soml => true,
            Saml => true,
            Vrfy => true,
            Expn => true,
            Help => false,
            Noop => false,
            Quit => false,
            Turn => false,
        }
    }
}

impl fmt::Show for Command {
    /// Format SMTP command display
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), io::IoError> {
        f.buf.write(match *self {
            Ehlo => "EHLO",
            Helo => "Helo",
            Mail => "MAIL FROM:",
            Rcpt => "RCPT TO:",
            Data => "DATA",
            Rset => "RSET",
            Send => "SEND TO:",
            Soml => "SOML TO:",
            Saml => "SAML TO:",
            Vrfy => "VRFY",
            Expn => "EXPN",
            Help => "HELP",
            Noop => "NOOP",
            Quit => "QUIT",
            Turn => "TURN"
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
    /// 8BITMIME keyword
    /// RFC 6152 : https://tools.ietf.org/html/rfc6152
    EightBitMime,
}

impl fmt::Show for EhloKeyword {
    /// Format SMTP response display
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), IoError> {
        f.buf.write(
            match self {
                &EightBitMime  => "8BITMIME".as_bytes()
            }
        )
    }
}

impl from_str::FromStr for EhloKeyword {
    // Match keywords
    fn from_str(s: &str) -> Option<EhloKeyword> {
        match s {
            "8BITMIME" => Some(EightBitMime),
            _          => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{SmtpCommand, EhloKeyword};

    #[test]
    fn test_command_parameters() {
        assert!((super::Help).takes_argument() == true);
        assert!((super::Rset).takes_argument() == false);
        assert!((super::Helo).needs_argument() == true);
    }

    #[test]
    fn test_command_to_str() {
        assert!(super::Turn.to_str() == ~"TURN");
    }

    #[test]
    fn test_command_fmt() {
        assert!(format!("{}", super::Turn) == ~"TURN");
    }

    #[test]
    fn test_get_simple_command() {
        assert!(SmtpCommand::new(super::Turn, None).to_str() == ~"TURN");
    }

    #[test]
    fn test_get_argument_command() {
        assert!(SmtpCommand::new(super::Ehlo, Some(~"example.example")).to_str() == ~"EHLO example.example");
    }

    #[test]
    fn test_ehlokeyword_fmt() {
        assert!(format!("{}", super::EightBitMime) == ~"8BITMIME");
    }

    #[test]
    fn test_ehlokeyword_from_str() {
        assert!(from_str::<EhloKeyword>("8BITMIME") == Some(super::EightBitMime));
    }
}
