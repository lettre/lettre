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
//! Will log the line:
//!
//! ```text
//! b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
//! ```

use EmailTransport;
use SendableEmail;
use smtp::error::{Error, SmtpResult};
use smtp::response::{Code, Response};
use std::io::Read;
use std::str::FromStr;

/// This transport logs the message envelope and returns the given response
#[derive(Debug)]
pub struct StubEmailTransport {
    response: Response,
}

impl StubEmailTransport {
    /// Creates a new transport that always returns the given response
    pub fn new(response: Response) -> StubEmailTransport {
        StubEmailTransport { response: response }
    }

    /// Creates a new transport that always returns a success response
    pub fn new_positive() -> StubEmailTransport {
        StubEmailTransport {
            response: Response::new(Code::from_str("200").unwrap(), vec!["ok".to_string()]),
        }
    }
}

/// SMTP result type
pub type StubResult = SmtpResult;

impl<'a, T: Read + 'a> EmailTransport<'a, T, StubResult> for StubEmailTransport {
    fn send<U: SendableEmail<'a, T>>(&mut self, email: &'a U) -> StubResult {

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
}
