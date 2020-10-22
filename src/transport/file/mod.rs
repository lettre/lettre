//! The file transport writes the emails to the given directory. The name of the file will be
//! `message_id.json`.
//! It can be useful for testing purposes, or if you want to keep track of sent messages.
//!
//! ## Sync example
//!
//! ```rust
//! use std::env::temp_dir;
//! use lettre::{Transport, Message, FileTransport};
//!
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
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
//! ```
//!
//! ## Async tokio 0.2
//!
//! ```rust
//! # # use std::error::Error;
//! # #[cfg(feature = "tokio02")]
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
//! ```rust
//! # use std::error::Error;
//! # #[cfg(feature = "async-std1")]
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
//! ```json
//! {
//!   "envelope": {
//!     "forward_path": [
//!       "hei@domain.tld"
//!     ],
//!     "reverse_path": "nobody@domain.tld"
//!   },
//!   "raw_message": null,
//!   "message": "From: NoBody <nobody@domain.tld>\r\nReply-To: Yuin <yuin@domain.tld>\r\nTo: Hei <hei@domain.tld>\r\nSubject: Happy new year\r\nDate: Tue, 18 Aug 2020 22:50:17 GMT\r\n\r\nBe happy!"
//! }
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

impl FileTransport {
    fn send_raw_impl(
        &self,
        envelope: &Envelope,
        email: &[u8],
    ) -> Result<(Uuid, PathBuf, String), serde_json::Error> {
        let email_id = Uuid::new_v4();
        let file = self.path.join(format!("{}.json", email_id));

        let serialized = match str::from_utf8(email) {
            // Serialize as UTF-8 string if possible
            Ok(m) => serde_json::to_string(&SerializableEmail {
                envelope: envelope.clone(),
                message: Some(m),
                raw_message: None,
            }),
            Err(_) => serde_json::to_string(&SerializableEmail {
                envelope: envelope.clone(),
                message: None,
                raw_message: Some(email),
            }),
        }?;

        Ok((email_id, file, serialized))
    }
}

impl Transport for FileTransport {
    type Ok = Id;
    type Error = Error;

    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use std::fs;

        let (email_id, file, serialized) = self.send_raw_impl(envelope, email)?;

        fs::write(file, serialized)?;
        Ok(email_id.to_string())
    }
}

#[cfg(feature = "async-std1")]
#[async_trait]
impl AsyncStd1Transport for FileTransport {
    type Ok = Id;
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use async_std::fs;

        let (email_id, file, serialized) = self.send_raw_impl(envelope, email)?;

        fs::write(file, serialized).await?;
        Ok(email_id.to_string())
    }
}

#[cfg(feature = "tokio02")]
#[async_trait]
impl Tokio02Transport for FileTransport {
    type Ok = Id;
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use tokio02_crate::fs;

        let (email_id, file, serialized) = self.send_raw_impl(envelope, email)?;

        fs::write(file, serialized).await?;
        Ok(email_id.to_string())
    }
}

#[cfg(feature = "tokio03")]
#[async_trait]
impl Tokio03Transport for FileTransport {
    type Ok = Id;
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use tokio03_crate::fs;

        let (email_id, file, serialized) = self.send_raw_impl(envelope, email)?;

        fs::write(file, serialized).await?;
        Ok(email_id.to_string())
    }
}
