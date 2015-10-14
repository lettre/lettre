//! TODO

use transport::EmailTransport;
use email::SendableEmail;
use transport::error::EmailResult;

/// TODO
pub struct Mailer<T: EmailTransport> {
    transport: T,
}

impl<T: EmailTransport> Mailer<T> {
    /// TODO
    pub fn new(transport: T) -> Mailer<T> {
        Mailer { transport: transport }
    }

    /// TODO
    pub fn send<S: SendableEmail>(&mut self, email: S) -> EmailResult {
        self.transport.send(email.to_addresses(),
                            email.from_address(),
                            email.message(),
                            email.message_id())
    }

    /// TODO
    pub fn close(&mut self) {
        self.transport.close()
    }
}
