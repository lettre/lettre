//! The stub transport only logs message envelope and drops the content. It can be useful for
//! testing purposes.
//!
//! #### Stub Transport
//!
//! The stub transport returns provided result and drops the content. It can be useful for
//! testing purposes.
//!
//! ```rust
//! # #[cfg(feature = "builder")]
//! # {
//! use lettre::{transport::stub::StubTransport, Message, Transport};
//!
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse()?)
//!     .reply_to("Yuin <yuin@domain.tld>".parse()?)
//!     .to("Hei <hei@domain.tld>".parse()?)
//!     .subject("Happy new year")
//!     .body(String::from("Be happy!"))?;
//!
//! let mut sender = StubTransport::new_ok();
//! let result = sender.send(&email);
//! assert!(result.is_ok());
//! # Ok(())
//! # }
//! # }
//! ```

#[cfg(any(feature = "tokio02", feature = "tokio1", feature = "async-std1"))]
use crate::AsyncTransport;
use crate::{address::Envelope, Transport};
#[cfg(any(feature = "tokio02", feature = "tokio1", feature = "async-std1"))]
use async_trait::async_trait;
use std::{error::Error as StdError, fmt};

#[derive(Debug, Copy, Clone)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("stub error")
    }
}

impl StdError for Error {}

/// This transport logs the message envelope and returns the given response
#[derive(Debug, Clone, Copy)]
pub struct StubTransport {
    response: Result<(), Error>,
}

impl StubTransport {
    /// Creates a new transport that always returns the given Result
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

#[cfg(any(feature = "tokio02", feature = "tokio1", feature = "async-std1"))]
#[async_trait]
impl AsyncTransport for StubTransport {
    type Ok = ();
    type Error = Error;

    async fn send_raw(&self, _envelope: &Envelope, _email: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.response
    }
}
