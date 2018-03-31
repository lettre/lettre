//! The file transport writes the emails to the given directory. The name of the file will be
//! `message_id.txt`.
//! It can be useful for testing purposes, or if you want to keep track of sent messages.
//!

use EmailTransport;
use SendableEmail;
use SimpleSendableEmail;
use file::error::FileResult;
use serde_json;
use std::fs::File;
use std::io::Read;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub mod error;

/// Writes the content and the envelope information to a file
#[derive(Debug)]
pub struct FileEmailTransport {
    path: PathBuf,
}

impl FileEmailTransport {
    /// Creates a new transport to the given directory
    pub fn new<P: AsRef<Path>>(path: P) -> FileEmailTransport {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        FileEmailTransport { path: path_buf }
    }
}

impl<'a, T: Read + 'a> EmailTransport<'a, T, FileResult> for FileEmailTransport {
    fn send<U: SendableEmail<'a, T> + 'a>(&mut self, email: &'a U) -> FileResult {
        let mut file = self.path.clone();
        file.push(format!("{}.txt", email.message_id()));

        let mut f = File::create(file.as_path())?;

        let mut message_content = String::new();
        let _ = email.message().read_to_string(&mut message_content);

        let simple_email = SimpleSendableEmail::new_with_envelope(
            email.envelope().clone(),
            email.message_id().to_string(),
            message_content,
        );

        f.write_all(serde_json::to_string(&simple_email)?.as_bytes())?;

        Ok(())
    }
}
