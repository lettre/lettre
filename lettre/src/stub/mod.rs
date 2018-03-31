//! The stub transport only logs message envelope and drops the content. It can be useful for
//! testing purposes.
//!

use EmailTransport;
use SendableEmail;
use std::io::Read;

/// This transport logs the message envelope and returns the given response
#[derive(Debug)]
pub struct StubEmailTransport {
    response: StubResult,
}

impl StubEmailTransport {
    /// Creates a new transport that always returns the given response
    pub fn new(response: StubResult) -> StubEmailTransport {
        StubEmailTransport { response }
    }

    /// Creates a new transport that always returns a success response
    pub fn new_positive() -> StubEmailTransport {
        StubEmailTransport { response: Ok(()) }
    }
}

/// SMTP result type
pub type StubResult = Result<(), ()>;

impl<'a, T: Read + 'a> EmailTransport<'a, T, StubResult> for StubEmailTransport {
    fn send<U: SendableEmail<'a, T>>(&mut self, email: &'a U) -> StubResult {
        let envelope = email.envelope();
        info!(
            "{}: from=<{}> to=<{:?}>",
            email.message_id(),
            match envelope.from() {
                Some(address) => address.to_string(),
                None => "".to_string(),
            },
            envelope.to()
        );
        self.response
    }
}
