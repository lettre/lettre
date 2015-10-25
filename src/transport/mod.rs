//! TODO
pub mod smtp;
pub mod error;
pub mod stub;
// pub mod file;

use transport::error::EmailResult;

/// Transport method for emails
pub trait EmailTransport {
    /// Sends the email
    fn send(&mut self,
            to_addresses: Vec<String>,
            from_address: String,
            message: String,
            message_id: String)
            -> EmailResult;
    /// Close the transport explicitely
    fn close(&mut self);
}
