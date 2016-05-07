//! Represents an Email transport
pub mod smtp;
pub mod error;
pub mod stub;
pub mod file;

use transport::error::EmailResult;
use email::SendableEmail;

/// Transport method for emails
pub trait EmailTransport {
    /// Sends the email
    fn send<T: SendableEmail>(&mut self, email: T) -> EmailResult;
    /// Close the transport explicitly
    fn close(&mut self);
}
