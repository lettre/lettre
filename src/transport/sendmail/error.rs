//! Error and result type for sendmail transport

use crate::BoxError;
use std::{error::Error as StdError, fmt};

/// The Errors that may occur when sending an email over sendmail
pub struct Error {
    inner: Box<Inner>,
}

struct Inner {
    kind: Kind,
    source: Option<BoxError>,
}

impl Error {
    pub(crate) fn new<E>(kind: Kind, source: Option<E>) -> Error
    where
        E: Into<BoxError>,
    {
        Error {
            inner: Box::new(Inner {
                kind,
                source: source.map(Into::into),
            }),
        }
    }

    /// Returns true if the error is from client
    pub fn is_client(&self) -> bool {
        matches!(self.inner.kind, Kind::Client)
    }

    /// Returns true if the error comes from the response
    pub fn is_response(&self) -> bool {
        matches!(self.inner.kind, Kind::Response)
    }
}

#[derive(Debug)]
pub(crate) enum Kind {
    /// Error parsing a response
    Response,
    /// Internal client error
    Client,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("lettre::transport::sendmail::Error");

        builder.field("kind", &self.inner.kind);

        if let Some(ref source) = self.inner.source {
            builder.field("source", source);
        }

        builder.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.kind {
            Kind::Response => f.write_str("response error")?,
            Kind::Client => f.write_str("internal client error")?,
        };

        if let Some(ref e) = self.inner.source {
            write!(f, ": {}", e)?;
        }

        Ok(())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.inner.source.as_ref().map(|e| {
            let r: &(dyn std::error::Error + 'static) = &**e;
            r
        })
    }
}

pub(crate) fn response<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Response, Some(e))
}

pub(crate) fn client<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Client, Some(e))
}
