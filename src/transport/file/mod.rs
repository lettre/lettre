//! The file transport writes the emails to the given directory. The name of the file will be
//! `message_id.eml`.
//! It can be useful for testing purposes, or if you want to keep track of sent messages.
//!
//! ## Sync example
//!
//! ```rust
//! # use std::error::Error;
//! #
//! # #[cfg(all(feature = "file-transport", feature = "builder"))]
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use std::env::temp_dir;
//!
//! use lettre::{message::header::ContentType, FileTransport, Message, Transport};
//!
//! // Write to the local temp directory
//! let sender = FileTransport::new(temp_dir());
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .header(ContentType::TEXT_PLAIN)
//!     .body(String::from("Be happy!"))?;
//!
//! sender.send(&email)?;
//! # Ok(())
//! # }
//!
//! # #[cfg(not(all(feature = "file-transport", feature = "builder")))]
//! # fn main() {}
//! ```
//!
//! ## Sync example with envelope
//!
//! It is possible to also write the envelope content in a separate JSON file
//! by using the `with_envelope` builder. The JSON file will be written in the
//! target directory with same name and a `json` extension.
//!
//! ```rust
//! # use std::error::Error;
//! #
//! # #[cfg(all(feature = "file-transport-envelope", feature = "builder"))]
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use std::env::temp_dir;
//!
//! use lettre::{message::header::ContentType, FileTransport, Message, Transport};
//!
//! // Write to the local temp directory
//! let sender = FileTransport::with_envelope(temp_dir());
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .header(ContentType::TEXT_PLAIN)
//!     .body(String::from("Be happy!"))?;
//!
//! sender.send(&email)?;
//! # Ok(())
//! # }
//!
//! # #[cfg(not(all(feature = "file-transport-envelope", feature = "builder")))]
//! # fn main() {}
//! ```
//!
//! ## Async tokio 1.x
//!
//! ```rust,no_run
//! # use std::error::Error;
//! #
//! # #[cfg(all(feature = "tokio1", feature = "file-transport", feature = "builder"))]
//! # async fn run() -> Result<(), Box<dyn Error>> {
//! use std::env::temp_dir;
//!
//! use lettre::{
//!     message::header::ContentType, AsyncFileTransport, AsyncTransport, Message, Tokio1Executor,
//! };
//!
//! // Write to the local temp directory
//! let sender = AsyncFileTransport::<Tokio1Executor>::new(temp_dir());
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .header(ContentType::TEXT_PLAIN)
//!     .body(String::from("Be happy!"))?;
//!
//! sender.send(email).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Async async-std 1.x
//!
//! ```rust,no_run
//! # use std::error::Error;
//! #
//! # #[cfg(all(feature = "async-std1", feature = "file-transport", feature = "builder"))]
//! # async fn run() -> Result<(), Box<dyn Error>> {
//! use std::env::temp_dir;
//!
//! use lettre::{
//!     message::header::ContentType, AsyncFileTransport, AsyncStd1Executor, AsyncTransport,
//!     Message,
//! };
//!
//! // Write to the local temp directory
//! let sender = AsyncFileTransport::<AsyncStd1Executor>::new(temp_dir());
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .header(ContentType::TEXT_PLAIN)
//!     .body(String::from("Be happy!"))?;
//!
//! sender.send(email).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ---
//!
//! Example email content result
//!
//! ```eml
//! From: NoBody <nobody@domain.tld>
//! Reply-To: Yuin <yuin@domain.tld>
//! To: Hei <hei@domain.tld>
//! Subject: Happy new year
//! Content-Type: text/plain; charset=utf-8
//! Date: Tue, 18 Aug 2020 22:50:17 GMT
//!
//! Be happy!
//! ```
//!
//! Example envelope result
//!
//! ```json
//! {"forward_path":["hei@domain.tld"],"reverse_path":"nobody@domain.tld"}
//! ```

#[cfg(any(feature = "async-std1", feature = "tokio1"))]
use std::marker::PhantomData;
use std::{
    path::{Path, PathBuf},
    str,
};

#[cfg(any(feature = "async-std1", feature = "tokio1"))]
use async_trait::async_trait;
use uuid::Uuid;

pub use self::error::Error;
use crate::{address::Envelope, Transport};
#[cfg(any(feature = "async-std1", feature = "tokio1"))]
use crate::{AsyncTransport, Executor};

mod error;

type Id = String;

/// Writes the content and the envelope information to a file
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(docsrs, doc(cfg(feature = "file-transport")))]
pub struct FileTransport {
    path: PathBuf,
    #[cfg(feature = "file-transport-envelope")]
    save_envelope: bool,
}

/// Asynchronously writes the content and the envelope information to a file
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio1", feature = "async-std1"))))]
#[cfg(any(feature = "async-std1", feature = "tokio1"))]
pub struct AsyncFileTransport<E: Executor> {
    inner: FileTransport,
    marker_: PhantomData<E>,
}

impl FileTransport {
    /// Creates a new transport to the given directory
    ///
    /// Writes the email content in eml format.
    pub fn new<P: AsRef<Path>>(path: P) -> FileTransport {
        FileTransport {
            path: PathBuf::from(path.as_ref()),
            #[cfg(feature = "file-transport-envelope")]
            save_envelope: false,
        }
    }

    /// Creates a new transport to the given directory
    ///
    /// Writes the email content in eml format and the envelope
    /// in json format.
    #[cfg(feature = "file-transport-envelope")]
    pub fn with_envelope<P: AsRef<Path>>(path: P) -> FileTransport {
        FileTransport {
            path: PathBuf::from(path.as_ref()),
            #[cfg(feature = "file-transport-envelope")]
            save_envelope: true,
        }
    }

    /// Read a message that was written using the file transport.
    ///
    /// Reads the envelope and the raw message content.
    #[cfg(feature = "file-transport-envelope")]
    pub fn read(&self, email_id: &str) -> Result<(Envelope, Vec<u8>), Error> {
        use std::fs;

        let eml_file = self.path.join(format!("{email_id}.eml"));
        let eml = fs::read(eml_file).map_err(error::io)?;

        let json_file = self.path.join(format!("{email_id}.json"));
        let json = fs::read(json_file).map_err(error::io)?;
        let envelope = serde_json::from_slice(&json).map_err(error::envelope)?;

        Ok((envelope, eml))
    }

    fn path(&self, email_id: &Uuid, extension: &str) -> PathBuf {
        self.path.join(format!("{email_id}.{extension}"))
    }
}

#[cfg(any(feature = "async-std1", feature = "tokio1"))]
impl<E> AsyncFileTransport<E>
where
    E: Executor,
{
    /// Creates a new transport to the given directory
    ///
    /// Writes the email content in eml format.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            inner: FileTransport::new(path),
            marker_: PhantomData,
        }
    }

    /// Creates a new transport to the given directory
    ///
    /// Writes the email content in eml format and the envelope
    /// in json format.
    #[cfg(feature = "file-transport-envelope")]
    pub fn with_envelope<P: AsRef<Path>>(path: P) -> Self {
        Self {
            inner: FileTransport::with_envelope(path),
            marker_: PhantomData,
        }
    }

    /// Read a message that was written using the file transport.
    ///
    /// Reads the envelope and the raw message content.
    #[cfg(feature = "file-transport-envelope")]
    pub async fn read(&self, email_id: &str) -> Result<(Envelope, Vec<u8>), Error> {
        let eml_file = self.inner.path.join(format!("{email_id}.eml"));
        let eml = E::fs_read(&eml_file).await.map_err(error::io)?;

        let json_file = self.inner.path.join(format!("{email_id}.json"));
        let json = E::fs_read(&json_file).await.map_err(error::io)?;
        let envelope = serde_json::from_slice(&json).map_err(error::envelope)?;

        Ok((envelope, eml))
    }
}

#[cfg(any(feature = "async-std1", feature = "tokio1"))]
impl<E: Executor> Clone for AsyncFileTransport<E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker_: PhantomData,
        }
    }
}

impl Transport for FileTransport {
    type Ok = Id;
    type Error = Error;

    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        use std::fs;

        let email_id = Uuid::new_v4();

        let file = self.path(&email_id, "eml");
        #[cfg(feature = "tracing")]
        tracing::debug!(?file, "writing email to");
        fs::write(file, email).map_err(error::io)?;

        #[cfg(feature = "file-transport-envelope")]
        {
            if self.save_envelope {
                let file = self.path(&email_id, "json");
                let buf = serde_json::to_string(&envelope).map_err(error::envelope)?;
                fs::write(file, buf).map_err(error::io)?;
            }
        }
        // use envelope anyway
        let _ = envelope;

        Ok(email_id.to_string())
    }
}

#[cfg(any(feature = "async-std1", feature = "tokio1"))]
#[async_trait]
impl<E> AsyncTransport for AsyncFileTransport<E>
where
    E: Executor,
{
    type Ok = Id;
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        let email_id = Uuid::new_v4();

        let file = self.inner.path(&email_id, "eml");
        #[cfg(feature = "tracing")]
        tracing::debug!(?file, "writing email to");
        E::fs_write(&file, email).await.map_err(error::io)?;

        #[cfg(feature = "file-transport-envelope")]
        {
            if self.inner.save_envelope {
                let file = self.inner.path(&email_id, "json");
                let buf = serde_json::to_vec(&envelope).map_err(error::envelope)?;
                E::fs_write(&file, &buf).await.map_err(error::io)?;
            }
        }
        // use envelope anyway
        let _ = envelope;

        Ok(email_id.to_string())
    }
}
