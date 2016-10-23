//! This transport uilizes the sendmail executable for each email.

use email::SendableEmail;
use std::error::Error;
use std::io::prelude::*;
use std::process::{Command, Stdio};

use transport::EmailTransport;
use transport::sendmail::error::SendmailResult;

pub mod error;

/// Writes the content and the envelope information to a file
pub struct SendmailTransport;

impl EmailTransport<SendmailResult> for SendmailTransport {
    fn send<T: SendableEmail>(&mut self, email: T) -> SendmailResult {
        // Spawn the `wc` command
        let process = try!(Command::new("/usr/sbin/sendmail")
            .args(&email.to_addresses())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn());

        match process.stdin.unwrap().write_all(email.message().clone().as_bytes()) {
            Err(why) => error!("couldn't write to sendmail stdin: {}",
                               why.description()),
            Ok(_) => info!("sent pangram to sendmail"),
        }

        let mut s = String::new();
        match process.stdout.unwrap().read_to_string(&mut s) {
            Err(why) => error!("couldn't read sendmail stdout: {}",
                               why.description()),
            Ok(_) => info!("sendmail responded with:\n{}", s),
        }
        
        Ok(())
    }

    fn close(&mut self) {
        ()
    }
}
