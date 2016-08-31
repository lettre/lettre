//! Represents an Email transport
pub mod smtp;
pub mod stub;
pub mod file;

use email::SendableEmail;

/// Transport method for emails
pub trait EmailTransport<U> {
    /// Sends the email
    fn send<T: SendableEmail>(&mut self, email: T) -> U;
    /// Close the transport explicitly
    fn close(&mut self);
}
