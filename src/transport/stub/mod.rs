//! The stub transport only logs message envelope and drops the content. It can be useful for
//! testing purposes.
//!
//! #### Stub Transport
//!
//! The stub transport returns provided result and drops the content. It can be useful for
//! testing purposes.
//!
//! ```rust
//! use lettre::{Message, Envelope, Transport, StubTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse().unwrap())
//!     .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
//!     .to("Hei <hei@domain.tld>".parse().unwrap())
//!     .subject("Happy new year")
//!     .body("Be happy!")
//!     .unwrap();
//!
//! let mut sender = StubTransport::new_ok();
//! let result = sender.send(&email);
//! assert!(result.is_ok());
//! ```

use crate::{Envelope, Transport};
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stub error")
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

/// This transport logs the message envelope and returns the given response
#[derive(Debug, Clone, Copy)]
pub struct StubTransport {
    response: Result<(), Error>,
}

impl StubTransport {
    /// Creates aResult new transport that always returns the given response
    pub fn new(response: Result<(), Error>) -> StubTransport {
        StubTransport { response }
    }

    /// Creates a new transport that always returns a success response
    pub fn new_ok() -> StubTransport {
        StubTransport { response: Ok(()) }
    }

    /// Creates a new transport that always returns an error
    pub fn new_error() -> StubTransport {
        StubTransport {
            response: Err(Error),
        }
    }
}

impl Transport for StubTransport {
    type Ok = ();
    type Error = Error;

    fn send_raw(&self, _envelope: &Envelope, _email: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.response
    }
}
