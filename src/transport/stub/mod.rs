//! This transport is a stub that only logs the message, and always returns
//! success

use email::SendableEmail;
use transport::EmailTransport;

pub mod error;

/// This transport does nothing except logging the message envelope
pub struct StubEmailTransport;

/// SMTP result type
pub type StubResult = Result<(), error::Error>;

impl EmailTransport<StubResult> for StubEmailTransport {
    fn send<T: SendableEmail>(&mut self, email: T) -> StubResult {

        info!("{}: from=<{}> to=<{:?}>",
              email.message_id(),
              email.from_address(),
              email.to_addresses());
        Ok(())
    }

    fn close(&mut self) {
        ()
    }
}
