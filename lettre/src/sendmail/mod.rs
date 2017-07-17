//! The sendmail transport sends the email using the local sendmail command.
//!
//! ```rust
//! use lettre::sendmail::SendmailTransport;
//! use lettre::{SimpleSendableEmail, EmailTransport, EmailAddress};
//!
//! let email = SimpleSendableEmail::new(
//!                 EmailAddress::new("user@localhost".to_string()),
//!                 vec![EmailAddress::new("root@localhost".to_string())],
//!                 "message_id".to_string(),
//!                 "Hello world".to_string(),
//!             );
//!
//! let mut sender = SendmailTransport::new();
//! let result = sender.send(email);
//! assert!(result.is_ok());
//! ```

use EmailTransport;
use SendableEmail;
use sendmail::error::SendmailResult;
use std::io::prelude::*;
use std::process::{Command, Stdio};

pub mod error;

/// Sends an email using the `sendmail` command
#[derive(Debug, Default)]
pub struct SendmailTransport {
    command: String,
}

impl SendmailTransport {
    /// Creates a new transport with the default `/usr/sbin/sendmail` command
    pub fn new() -> SendmailTransport {
        SendmailTransport { command: "/usr/sbin/sendmail".to_string() }
    }

    /// Creates a new transport to the given sendmail command
    pub fn new_with_command<S: Into<String>>(command: S) -> SendmailTransport {
        SendmailTransport { command: command.into() }
    }
}

impl EmailTransport<SendmailResult> for SendmailTransport {
    fn send<T: SendableEmail>(&mut self, email: T) -> SendmailResult {
        // Spawn the sendmail command
        let to_addresses: Vec<String> = email.to().iter().map(|x| x.to_string()).collect();
        let mut process = Command::new(&self.command)
            .args(
                &[
                    "-i",
                    "-f",
                    &email.from().to_string(),
                    &to_addresses.join(" "),
                ],
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        match process.stdin.as_mut().unwrap().write_all(
            email.message().as_bytes(),
        ) {
            Ok(_) => (),
            Err(error) => return Err(From::from(error)),
        }

        info!("Wrote message to stdin");

        if let Ok(output) = process.wait_with_output() {
            if output.status.success() {
                Ok(())
            } else {
                Err(From::from("The message could not be sent"))
            }
        } else {
            Err(From::from("The sendmail process stopped"))
        }
    }

    fn close(&mut self) {
        ()
    }
}
