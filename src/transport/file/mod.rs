//! This transport creates a file for each email, containing the envelope information and the email
//! itself.

use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::fs::File;

use transport::EmailTransport;
use transport::error::EmailResult;
use transport::smtp::response::Response;
use transport::smtp::response::{Category, Code, Severity};
use email::SendableEmail;

/// Writes the content and the envelope information to a file
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

impl EmailTransport for FileEmailTransport {
    fn send<T: SendableEmail>(&mut self, email: T) -> EmailResult {
        let mut file = self.path.clone();
        file.push(format!("{}.txt", email.message_id()));

        let mut f = try!(File::create(file.as_path()));

        let log_line = format!("{}: from=<{}> to=<{}>\n",
                               email.message_id(),
                               email.from_address(),
                               email.to_addresses().join("> to=<"));

        try!(f.write_all(log_line.as_bytes()));
        try!(f.write_all(email.message().clone().as_bytes()));

        info!("{} status=<written>", log_line);

        Ok(Response::new(Code::new(Severity::PositiveCompletion, Category::MailSystem, 0),
                         vec![format!("Ok: email written to {}",
                                      file.to_str().unwrap_or("non-UTF-8 path"))]))
    }

    fn close(&mut self) {
        ()
    }
}
