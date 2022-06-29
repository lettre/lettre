//! The stub transport logs message envelopes as well as contents. It can be useful for testing
//! purposes.
//!
//! # Stub Transport
//!
//! The stub transport logs message envelopes as well as contents. It can be useful for testing
//! purposes.
//!
//! # Examples
//!
//! ```rust
//! # #[cfg(feature = "builder")]
//! # {
//! use lettre::{transport::stub::StubTransport, Message, Transport};
//!
//! # use std::error::Error;
//! # fn try_main() -> Result<(), Box<dyn Error>> {
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
//! assert_eq!(
//!     sender.messages(),
//!     vec![(
//!         email.envelope().clone(),
//!         String::from_utf8(email.formatted()).unwrap()
//!     )],
//! );
//! # Ok(())
//! # }
//! # try_main().unwrap();
//! # }
//! ```

use std::{
    error::Error as StdError,
    fmt,
    sync::{Arc, Mutex as StdMutex},
};

#[cfg(any(feature = "tokio1", feature = "async-std1"))]
use async_trait::async_trait;
#[cfg(any(feature = "tokio1", feature = "async-std1"))]
use futures_util::lock::Mutex as FuturesMutex;

#[cfg(any(feature = "tokio1", feature = "async-std1"))]
use crate::AsyncTransport;
use crate::{address::Envelope, Transport};

/// An error returned by the stub transport
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("stub error")
    }
}

impl StdError for Error {}

/// This transport logs messages and always returns the given response
#[derive(Debug, Clone)]
pub struct StubTransport {
    response: Result<(), Error>,
    message_log: Arc<StdMutex<Vec<(Envelope, String)>>>,
}

/// This transport logs messages and always returns the given response
#[derive(Debug, Clone)]
#[cfg(any(feature = "tokio1", feature = "async-std1"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio1", feature = "async-std1"))))]
pub struct AsyncStubTransport {
    response: Result<(), Error>,
    message_log: Arc<FuturesMutex<Vec<(Envelope, String)>>>,
}

impl StubTransport {
    /// Creates a new transport that always returns the given Result
    pub fn new(response: Result<(), Error>) -> Self {
        Self {
            response,
            message_log: Arc::new(StdMutex::new(vec![])),
        }
    }

    /// Creates a new transport that always returns a success response
    pub fn new_ok() -> Self {
        Self {
            response: Ok(()),
            message_log: Arc::new(StdMutex::new(vec![])),
        }
    }

    /// Creates a new transport that always returns an error
    pub fn new_error() -> Self {
        Self {
            response: Err(Error),
            message_log: Arc::new(StdMutex::new(vec![])),
        }
    }

    /// Return all logged messages sent using [`Transport::send_raw`]
    pub fn messages(&self) -> Vec<(Envelope, String)> {
        self.message_log
            .lock()
            .expect("Couldn't acquire lock to write message log")
            .clone()
    }
}

#[cfg(any(feature = "async-std1", feature = "tokio1"))]
impl AsyncStubTransport {
    /// Creates a new transport that always returns the given Result
    pub fn new(response: Result<(), Error>) -> Self {
        Self {
            response,
            message_log: Arc::new(FuturesMutex::new(vec![])),
        }
    }

    /// Creates a new transport that always returns a success response
    pub fn new_ok() -> Self {
        Self {
            response: Ok(()),
            message_log: Arc::new(FuturesMutex::new(vec![])),
        }
    }

    /// Creates a new transport that always returns an error
    pub fn new_error() -> Self {
        Self {
            response: Err(Error),
            message_log: Arc::new(FuturesMutex::new(vec![])),
        }
    }

    /// Return all logged messages sent using [`AsyncTransport::send_raw`]
    #[cfg(any(feature = "tokio1", feature = "async-std1"))]
    pub async fn messages(&self) -> Vec<(Envelope, String)> {
        self.message_log.lock().await.clone()
    }
}

impl Transport for StubTransport {
    type Ok = ();
    type Error = Error;

    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.message_log
            .lock()
            .expect("Couldn't acquire lock to write message log")
            .push((envelope.clone(), String::from_utf8_lossy(email).into()));
        self.response
    }
}

#[cfg(any(feature = "tokio1", feature = "async-std1"))]
#[async_trait]
impl AsyncTransport for AsyncStubTransport {
    type Ok = ();
    type Error = Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.message_log
            .lock()
            .await
            .push((envelope.clone(), String::from_utf8_lossy(email).into()));
        self.response
    }
}
