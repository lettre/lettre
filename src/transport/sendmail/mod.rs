//! The sendmail transport sends the email using the local sendmail command.
//!
//! ## Sync example
//!
//! ```rust
//! # use std::error::Error;
//!
//! # #[cfg(all(feature = "sendmail-transport", feature = "builder"))]
//! # fn main() -> Result<(), Box<dyn Error>> {
//! # use lettre::{Message, Transport, SendmailTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body("Be happy!")?;
//!
//! let sender = SendmailTransport::new();
//! let result = sender.send(&email);
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//!
//! # #[cfg(not(all(feature = "sendmail-transport", feature = "builder")))]
//! # fn main() {}
//! ```
//!
//! ## Async tokio 0.2 example
//!
//! ```rust,no_run
//! # use std::error::Error;
//!
//! # #[cfg(all(feature = "tokio02", feature = "sendmail-transport", feature = "builder"))]
//! # async fn run() -> Result<(), Box<dyn Error>> {
//! use lettre::{Message, Tokio02Transport, SendmailTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body("Be happy!")?;
//!
//! let sender = SendmailTransport::new();
//! let result = sender.send(email).await;
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//! ```
//!
//! ## Async async-std 1.x example
//!
//!```rust,no_run
//! # use std::error::Error;
//!
//! # #[cfg(all(feature = "async-std1", feature = "sendmail-transport", feature = "builder"))]
//! # async fn run() -> Result<(), Box<dyn Error>> {
//! use lettre::{Message, AsyncStd1Transport, SendmailTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body("Be happy!")?;
//!
//! let sender = SendmailTransport::new();
//! let result = sender.send(email).await;
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//! ```

pub use self::error::Error;
use crate::address::Envelope;
#[cfg(feature = "async-std1")]
use crate::AsyncStd1Transport;
#[cfg(feature = "tokio02")]
use crate::Tokio02Transport;
#[cfg(feature = "tokio03")]
use crate::Tokio03Transport;
use crate::Transport;
#[cfg(any(feature = "async-std1", feature = "tokio02", feature = "tokio03"))]
use async_trait::async_trait;
use std::{
    ffi::OsString,
    io::prelude::*,
    process::{Command, Stdio},
};

mod error;

const DEFAUT_SENDMAIL: &str = "/usr/sbin/sendmail";

/// Sends an email using the `sendmail` command
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SendmailTransport {
    command: OsString,
}

impl SendmailTransport {
    /// Creates a new transport with the default `/usr/sbin/sendmail` command
    pub fn new() -> SendmailTransport {
        SendmailTransport {
            command: DEFAUT_SENDMAIL.into(),
        }
    }

    /// Creates a new transport to the given sendmail command
    pub fn new_with_command<S: Into<OsString>>(command: S) -> SendmailTransport {
        SendmailTransport {
            command: command.into(),
        }
    }

    fn command(&self, envelope: &Envelope) -> Command {
        let mut c = Command::new(&self.command);
        c.arg("-i");
        if let Some(from) = envelope.from() {
            c.arg("-f").arg(from);
        }
        c.arg("--")
            .args(envelope.to())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        c
    }

    #[cfg(feature = "tokio02")]
    fn tokio02_command(&self, envelope: &Envelope) -> tokio02_crate::process::Command {
        use tokio02_crate::process::Command;

        let mut c = Command::new(&self.command);
        c.kill_on_drop(true);
        c.arg("-i");
        if let Some(from) = envelope.from() {
            c.arg("-f").arg(from);
        }
        c.arg("--")
            .args(envelope.to())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        c
    }

    #[cfg(feature = "tokio03")]
    fn tokio03_command(&self, envelope: &Envelope) -> tokio03_crate::process::Command {
        use tokio03_crate::process::Command;

        let mut c = Command::new(&self.command);
        c.kill_on_drop(true);
        c.arg("-i");
        if let Some(from) = envelope.from() {
            c.arg("-f").arg(from);
        }
        c.arg("--")
            .args(envelope.to())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        c
    }

    #[cfg(feature = "async-std1")]
    fn async_std_command(&self, envelope: &Envelope) -> async_std::process::Command {
        use async_std::process::Command;

        let mut c = Command::new(&self.command);
        // TODO: figure out why enabling this kills it earlier
        // c.kill_on_drop(true);
        c.arg("-i");
        if let Some(from) = envelope.from() {
            c.arg("-f").arg(from);
        }
        c.arg("--")
            .args(envelope.to())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        c
    }
}

impl Default for SendmailTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for SendmailTransport {
    type Ok = ();
    type Error = Error;

    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        // Spawn the sendmail command
        let mut process = self.command(envelope).spawn()?;

        process.stdin.as_mut().unwrap().write_all(email)?;
        let output = process.wait_with_output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(error::Error::Client(String::from_utf8(output.stderr)?))
        }
    }
}

#[cfg(feature = "async-std1")]
#[async_trait]
impl AsyncStd1Transport for SendmailTransport {
    type Ok = ();
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use async_std::io::prelude::WriteExt;

        let mut command = self.async_std_command(envelope);

        // Spawn the sendmail command
        let mut process = command.spawn()?;

        process.stdin.as_mut().unwrap().write_all(&email).await?;
        let output = process.output().await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Client(String::from_utf8(output.stderr)?))
        }
    }
}

#[cfg(feature = "tokio02")]
#[async_trait]
impl Tokio02Transport for SendmailTransport {
    type Ok = ();
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use tokio02_crate::io::AsyncWriteExt;

        let mut command = self.tokio02_command(envelope);

        // Spawn the sendmail command
        let mut process = command.spawn()?;

        process.stdin.as_mut().unwrap().write_all(&email).await?;
        let output = process.wait_with_output().await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Client(String::from_utf8(output.stderr)?))
        }
    }
}

#[cfg(feature = "tokio03")]
#[async_trait]
impl Tokio03Transport for SendmailTransport {
    type Ok = ();
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use tokio03_crate::io::AsyncWriteExt;

        let mut command = self.tokio03_command(envelope);

        // Spawn the sendmail command
        let mut process = command.spawn()?;

        process.stdin.as_mut().unwrap().write_all(&email).await?;
        let output = process.wait_with_output().await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Client(String::from_utf8(output.stderr)?))
        }
    }
}
