//! The sendmail transport sends the email using the local sendmail command.
//!

use {EmailTransport, SendableEmail};
use sendmail::error::SendmailResult;
use std::io::Read;
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
        SendmailTransport {
            command: "/usr/sbin/sendmail".to_string(),
        }
    }

    /// Creates a new transport to the given sendmail command
    pub fn new_with_command<S: Into<String>>(command: S) -> SendmailTransport {
        SendmailTransport {
            command: command.into(),
        }
    }
}

impl<'a, T: Read + 'a> EmailTransport<'a, T, SendmailResult> for SendmailTransport {
    fn send<U: SendableEmail<'a, T> + 'a>(&mut self, email: &'a U) -> SendmailResult {
        let envelope = email.envelope();

        // Spawn the sendmail command
        let to_addresses: Vec<String> = envelope.to().iter().map(|x| x.to_string()).collect();
        let mut process = Command::new(&self.command)
            .args(&[
                "-i",
                "-f",
                &match envelope.from() {
                    Some(address) => address.to_string(),
                    None => "\"\"".to_string(),
                },
                &to_addresses.join(" "),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut message_content = String::new();
        let _ = email.message().read_to_string(&mut message_content);

        match process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(message_content.as_bytes())
        {
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
}
