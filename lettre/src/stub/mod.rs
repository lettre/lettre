//! The stub transport only logs message envelope and drops the content. It can be useful for
//! testing purposes.
//!
//! ```rust
//! use lettre::stub::StubEmailTransport;
//! use lettre::{SimpleSendableEmail, EmailTransport};
//!
//! let email = SimpleSendableEmail::new(
//!                 "user@localhost",
//!                 vec!["root@localhost"],
//!                 "message_id",
//!                 "Hello world"
//!             );
//!
//! let mut sender = StubEmailTransport;
//! let result = sender.send(email);
//! assert!(result.is_ok());
//! ```
//!
//! Will log the line:
//!
//! ```text
//! b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
//! ```

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

        info!(
            "{}: from=<{}> to=<{:?}>",
            email.message_id(),
            email.from(),
            email.to()
        );
        Ok(())
    }

    fn close(&mut self) {
        ()
    }
}
