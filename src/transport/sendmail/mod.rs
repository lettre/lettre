//! The sendmail transport sends the email using the local `sendmail` command.
//!
//! ## Sync example
//!
//! ```rust
//! # use std::error::Error;
//! #
//! # #[cfg(all(feature = "sendmail-transport", feature = "builder"))]
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use lettre::{Message, SendmailTransport, Transport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body(String::from("Be happy!"))?;
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
//! #
//! # #[cfg(all(feature = "tokio02", feature = "sendmail-transport", feature = "builder"))]
//! # async fn run() -> Result<(), Box<dyn Error>> {
//! use lettre::{Message, AsyncTransport, Tokio02Executor, AsyncSendmailTransport, SendmailTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body(String::from("Be happy!"))?;
//!
//! let sender = AsyncSendmailTransport::<Tokio02Executor>::new();
//! let result = sender.send(email).await;
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//! ```
//!
//! ## Async tokio 1.x example
//!
//! ```rust,no_run
//! # use std::error::Error;
//! #
//! # #[cfg(all(feature = "tokio1", feature = "sendmail-transport", feature = "builder"))]
//! # async fn run() -> Result<(), Box<dyn Error>> {
//! use lettre::{Message, AsyncTransport, Tokio1Executor, AsyncSendmailTransport, SendmailTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body(String::from("Be happy!"))?;
//!
//! let sender = AsyncSendmailTransport::<Tokio1Executor>::new();
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
//! #
//! # #[cfg(all(feature = "async-std1", feature = "sendmail-transport", feature = "builder"))]
//! # async fn run() -> Result<(), Box<dyn Error>> {
//! use lettre::{Message, AsyncTransport, AsyncStd1Executor, AsyncSendmailTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body(String::from("Be happy!"))?;
//!
//! let sender = AsyncSendmailTransport::<AsyncStd1Executor>::new();
//! let result = sender.send(email).await;
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//! ```

pub use self::error::Error;
#[cfg(feature = "async-std1")]
use crate::AsyncStd1Executor;
#[cfg(feature = "tokio02")]
use crate::Tokio02Executor;
#[cfg(feature = "tokio1")]
use crate::Tokio1Executor;
use crate::{address::Envelope, Transport};
#[cfg(any(feature = "async-std1", feature = "tokio02", feature = "tokio1"))]
use crate::{AsyncTransport, Executor};
#[cfg(any(feature = "async-std1", feature = "tokio02", feature = "tokio1"))]
use async_trait::async_trait;
#[cfg(any(feature = "async-std1", feature = "tokio02", feature = "tokio1"))]
use std::marker::PhantomData;
use std::{
    ffi::OsString,
    io::prelude::*,
    process::{Command, Stdio},
};

mod error;

const DEFAUT_SENDMAIL: &str = "/usr/sbin/sendmail";

/// Sends emails using the `sendmail` command
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(docsrs, doc(cfg(feature = "sendmail-transport")))]
pub struct SendmailTransport {
    command: OsString,
}

/// Asynchronously sends emails using the `sendmail` command
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg(any(feature = "async-std1", feature = "tokio02", feature = "tokio1"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "tokio02", feature = "tokio1", feature = "async-std1")))
)]
pub struct AsyncSendmailTransport<E: Executor> {
    inner: SendmailTransport,
    marker_: PhantomData<E>,
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
}

#[cfg(any(feature = "async-std1", feature = "tokio02", feature = "tokio1"))]
impl<E> AsyncSendmailTransport<E>
where
    E: Executor,
{
    /// Creates a new transport with the default `/usr/sbin/sendmail` command
    pub fn new() -> Self {
        Self {
            inner: SendmailTransport::new(),
            marker_: PhantomData,
        }
    }

    /// Creates a new transport to the given sendmail command
    pub fn new_with_command<S: Into<OsString>>(command: S) -> Self {
        Self {
            inner: SendmailTransport::new_with_command(command),
            marker_: PhantomData,
        }
    }

    #[cfg(feature = "tokio02")]
    fn tokio02_command(&self, envelope: &Envelope) -> tokio02_crate::process::Command {
        use tokio02_crate::process::Command;

        let mut c = Command::new(&self.inner.command);
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

    #[cfg(feature = "tokio1")]
    fn tokio1_command(&self, envelope: &Envelope) -> tokio1_crate::process::Command {
        use tokio1_crate::process::Command;

        let mut c = Command::new(&self.inner.command);
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

        let mut c = Command::new(&self.inner.command);
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

#[cfg(any(feature = "async-std1", feature = "tokio02", feature = "tokio1"))]
impl<E> Default for AsyncSendmailTransport<E>
where
    E: Executor,
{
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for SendmailTransport {
    type Ok = ();
    type Error = Error;

    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        // Spawn the sendmail command
        let mut process = self.command(envelope).spawn().map_err(Error::Io)?;

        process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(email)
            .map_err(Error::Io)?;
        let output = process.wait_with_output().map_err(Error::Io)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8(output.stderr).map_err(Error::Utf8Parsing)?;
            Err(Error::Client(stderr))
        }
    }
}

#[cfg(feature = "async-std1")]
#[async_trait]
impl AsyncTransport for AsyncSendmailTransport<AsyncStd1Executor> {
    type Ok = ();
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use async_std::io::prelude::WriteExt;

        let mut command = self.async_std_command(envelope);

        // Spawn the sendmail command
        let mut process = command.spawn().map_err(Error::Io)?;

        process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(&email)
            .await
            .map_err(Error::Io)?;
        let output = process.output().await.map_err(Error::Io)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8(output.stderr).map_err(Error::Utf8Parsing)?;
            Err(Error::Client(stderr))
        }
    }
}

#[cfg(feature = "tokio02")]
#[async_trait]
impl AsyncTransport for AsyncSendmailTransport<Tokio02Executor> {
    type Ok = ();
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use tokio02_crate::io::AsyncWriteExt;

        let mut command = self.tokio02_command(envelope);

        // Spawn the sendmail command
        let mut process = command.spawn().map_err(Error::Io)?;

        process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(&email)
            .await
            .map_err(Error::Io)?;
        let output = process.wait_with_output().await.map_err(Error::Io)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8(output.stderr).map_err(Error::Utf8Parsing)?;
            Err(Error::Client(stderr))
        }
    }
}

#[cfg(feature = "tokio1")]
#[async_trait]
impl AsyncTransport for AsyncSendmailTransport<Tokio1Executor> {
    type Ok = ();
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use tokio1_crate::io::AsyncWriteExt;

        let mut command = self.tokio1_command(envelope);

        // Spawn the sendmail command
        let mut process = command.spawn().map_err(Error::Io)?;

        process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(&email)
            .await
            .map_err(Error::Io)?;
        let output = process.wait_with_output().await.map_err(Error::Io)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8(output.stderr).map_err(Error::Utf8Parsing)?;
            Err(Error::Client(stderr))
        }
    }
}
