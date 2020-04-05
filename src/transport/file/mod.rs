//! The file transport writes the emails to the given directory. The name of the file will be
//! `message_id.txt`.
//! It can be useful for testing purposes, or if you want to keep track of sent messages.
//!

use crate::{transport::file::error::FileResult, Envelope, Message, Transport};
use serde_json;
use std::{
    fmt::Display,
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub mod error;

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
struct SerializableEmail {
    envelope: Envelope,
    message: Vec<u8>,
}

impl<'a, B> Transport<'a, B> for FileTransport {
    type Result = FileResult;

    fn send(&mut self, email: Message<B>) -> Self::Result
    where
        B: Display,
    {
        let email_id = Uuid::new_v4();

        let mut file = self.path.clone();
        file.push(format!("{}.json", email_id));

        let serialized = serde_json::to_string(&SerializableEmail {
            envelope: email.envelope().clone(),
            message: email.to_string().into_bytes(),
        })?;

        File::create(file.as_path())?.write_all(serialized.as_bytes())?;
        Ok(email_id.to_string())
    }
}
