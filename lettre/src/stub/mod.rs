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
//! let mut sender = StubEmailTransport::new_positive();
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
use smtp::response::{Code, Response};
use smtp::error::{Error, SmtpResult};
use std::str::FromStr;

/// This transport logs the message envelope and returns the given response
#[derive(Debug)]
pub struct StubEmailTransport {
    response: Response,
}

impl StubEmailTransport {
    /// Creates a new transport that always returns the given response
    pub fn new(response: Response) -> StubEmailTransport {
        StubEmailTransport {
            response: response,
        }
    }

    /// Creates a new transport that always returns a success response
    pub fn new_positive() -> StubEmailTransport {
        StubEmailTransport {
            response: Response::new(Code::from_str("200").unwrap(), vec!["ok".to_string()])
        }
    }
}

/// SMTP result type
pub type StubResult = SmtpResult;

impl EmailTransport<StubResult> for StubEmailTransport {
    fn send<T: SendableEmail>(&mut self, email: T) -> StubResult {

        info!(
            "{}: from=<{}> to=<{:?}>",
            email.message_id(),
            email.from(),
            email.to()
        );
        if self.response.is_positive() {
            Ok(self.response.clone())
        } else {
            Err(Error::from(self.response.clone()))
        }
    }

    fn close(&mut self) {
        ()
    }
}
