//! This transport is a stub that only logs the message, and always returns
//! success

use EmailTransport;
use SendableEmail;

pub mod error;

/// This transport does nothing except logging the message envelope
#[derive(Debug)]
pub struct StubEmailTransport;

/// SMTP result type
pub type StubResult = Result<(), error::Error>;

impl EmailTransport<StubResult> for StubEmailTransport {
    fn send<T: SendableEmail>(&mut self, email: T) -> StubResult {

        info!("{}: from=<{}> to=<{:?}>",
              email.message_id(),
              email.from(),
              email.to());
        Ok(())
    }

    fn close(&mut self) {
        ()
    }
}
