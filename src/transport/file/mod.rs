//! The file transport writes the emails to the given directory. The name of the file will be
//! `message_id.eml`.
//! It can be useful for testing purposes, or if you want to keep track of sent messages.
//!
//! ## Sync example
//!
//! ```rust
//! # use std::error::Error;
//!
//! # #[cfg(all(feature = "file-transport", feature = "builder"))]
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use std::env::temp_dir;
//! use lettre::{Transport, Message, FileTransport};
//!
//! // Write to the local temp directory
//! let sender = FileTransport::new(temp_dir());
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body("Be happy!")?;
//!
//! let result = sender.send(&email);
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//!
//! # #[cfg(not(all(feature = "file-transport", feature = "builder")))]
//! # fn main() {}
//! ```
//!
//!
//! ## Async tokio 0.2
//!
//! ```rust,no_run
//! # use std::error::Error;
//!
//! # #[cfg(all(feature = "tokio02", feature = "file-transport", feature = "builder"))]
//! # async fn run() -> Result<(), Box<dyn Error>> {
//! use std::env::temp_dir;
//! use lettre::{Tokio02Transport, Message, FileTransport};
//!
//! // Write to the local temp directory
//! let sender = FileTransport::new(temp_dir());
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body("Be happy!")?;
//!
//! let result = sender.send(email).await;
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//! ```
//!
//! ## Async async-std 1.x
//!
//! ```rust,no_run
//! # use std::error::Error;
//!
//! # #[cfg(all(feature = "async-std1", feature = "file-transport", feature = "builder"))]
//! # async fn run() -> Result<(), Box<dyn Error>> {
//! use std::env::temp_dir;
//! use lettre::{AsyncStd1Transport, Message, FileTransport};
//!
//! // Write to the local temp directory
//! let sender = FileTransport::new(temp_dir());
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body("Be happy!")?;
//!
//! let result = sender.send(email).await;
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//! ```
//!
//! ---
//!
//! Example result
//!
//! ```eml
//! From: NoBody <nobody@domain.tld>
//! Reply-To: Yuin <yuin@domain.tld>
//! To: Hei <hei@domain.tld>
//! Subject: Happy new year
//! Date: Tue, 18 Aug 2020 22:50:17 GMT
//!
//! Be happy!
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
    path::{Path, PathBuf},
    str,
};
use uuid::Uuid;

mod error;

type Id = String;

/// Writes the content and the envelope information to a file
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileTransport {
    path: PathBuf,
}

impl FileTransport {
    /// Creates a new transport to the given directory
    pub fn new<P: AsRef<Path>>(path: P) -> FileTransport {
        FileTransport {
            path: PathBuf::from(path.as_ref()),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct SerializableEmail<'a> {
    envelope: Envelope,
    raw_message: Option<&'a [u8]>,
    message: Option<&'a str>,
}

impl Transport for FileTransport {
    type Ok = Id;
    type Error = Error;

    fn send_raw(&self, _envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use std::fs;

        let email_id = Uuid::new_v4();
        let file = self.path.join(format!("{}.eml", email_id));

        fs::write(file, email)?;
        Ok(email_id.to_string())
    }
}

#[cfg(feature = "async-std1")]
#[async_trait]
impl AsyncStd1Transport for FileTransport {
    type Ok = Id;
    type Error = Error;

    async fn send_raw(&self, _envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use async_std::fs;

        let email_id = Uuid::new_v4();
        let file = self.path.join(format!("{}.eml", email_id));

        fs::write(file, email).await?;
        Ok(email_id.to_string())
    }
}

#[cfg(feature = "tokio02")]
#[async_trait]
impl Tokio02Transport for FileTransport {
    type Ok = Id;
    type Error = Error;

    async fn send_raw(&self, _envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use tokio02_crate::fs;

        let email_id = Uuid::new_v4();
        let file = self.path.join(format!("{}.eml", email_id));

        fs::write(file, email).await?;
        Ok(email_id.to_string())
    }
}

#[cfg(feature = "tokio03")]
#[async_trait]
impl Tokio03Transport for FileTransport {
    type Ok = Id;
    type Error = Error;

    async fn send_raw(&self, _envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use tokio03_crate::fs;

        let email_id = Uuid::new_v4();
        let file = self.path.join(format!("{}.eml", email_id));

        fs::write(file, email).await?;
        Ok(email_id.to_string())
    }
}
