//! The stub transport only logs message envelope and drops the content. It can be useful for
//! testing purposes.
//!
//! ```rust
//! use lettre::stub::StubEmailTransport;
//! use lettre::{SimpleSendableEmail, EmailTransport, EmailAddress};
//!
//! let email = SimpleSendableEmail::new(
//!                 EmailAddress::new("user@localhost".to_string()),
//!                 vec![EmailAddress::new("root@localhost".to_string())],
//!                 "message_id".to_string(),
//!                 "Hello world".to_string(),
//!             );
//!
//! let mut sender = StubEmailTransport::new_positive();
//! let result = sender.send(&email);
//! assert!(result.is_ok());
//! ```
//!
//! Will log (when using a logger like `env_logger`):
//!
//! ```text
//! b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
//! ```

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
        StubEmailTransport { response: response }
    }

    /// Creates a new transport that always returns a success response
    pub fn new_positive() -> StubEmailTransport {
        StubEmailTransport {
            response: Ok(()),
        }
    }
}

/// SMTP result type
pub type StubResult = Result<(), ()>;

impl<'a, T: Read + 'a> EmailTransport<'a, T, StubResult> for StubEmailTransport {
    fn send<U: SendableEmail<'a, T>>(&mut self, email: &'a U) -> StubResult {

        info!(
            "{}: from=<{}> to=<{:?}>",
            email.message_id(),
            email.from(),
            email.to()
        );
        self.response
    }
}
