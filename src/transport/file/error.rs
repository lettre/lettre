//! Error and result type for file transport

use crate::BoxError;
use std::{error::Error as StdError, fmt};

/// The Errors that may occur when sending an email over SMTP
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

    /// Returns true if the error is a file I/O error
    pub fn is_io(&self) -> bool {
        matches!(self.inner.kind, Kind::Io)
    }

    /// Returns true if the error is an envelope serialization or deserialization error
    #[cfg(feature = "file-transport-envelope")]
    pub fn is_envelope(&self) -> bool {
        matches!(self.inner.kind, Kind::Envelope)
    }
}

#[derive(Debug)]
pub(crate) enum Kind {
    /// File I/O error
    Io,
    /// Envelope serialization/deserialization error
    #[cfg(feature = "file-transport-envelope")]
    Envelope,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("lettre::transport::file::Error");

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
            Kind::Io => f.write_str("response error")?,
            #[cfg(feature = "file-transport-envelope")]
            Kind::Envelope => f.write_str("internal client error")?,
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

pub(crate) fn io<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Io, Some(e))
}

#[cfg(feature = "file-transport-envelope")]
pub(crate) fn envelope<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Envelope, Some(e))
}
