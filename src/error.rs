use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

// FIXME message-specific errors
/// Error type for email content
#[derive(Debug)]
pub enum Error {
    /// Missing from in envelope
    MissingFrom,
    /// Missing to in envelope
    MissingTo,
    /// Can only be one from in envelope
    TooManyFrom,
    /// Invalid email: missing at
    EmailMissingAt,
    /// Invalid email: missing local part
    EmailMissingLocalPart,
    /// Invalid email: missing domain
    EmailMissingDomain,
    /// Cannot parse filename for attachment
    CannotParseFilename,
    /// IO error
    Io(std::io::Error),
    /// Non-ASCII chars
    NonAsciiChars,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::MissingFrom => f.write_str("missing source address, invalid envelope"),
            Error::MissingTo => f.write_str("missing destination address, invalid envelope"),
            Error::TooManyFrom => f.write_str("there can only be one source address"),
            Error::EmailMissingAt => f.write_str("missing @ in email address"),
            Error::EmailMissingLocalPart => f.write_str("missing local part in email address"),
            Error::EmailMissingDomain => f.write_str("missing domain in email address"),
            Error::CannotParseFilename => f.write_str("could not parse attachment filename"),
            Error::NonAsciiChars => f.write_str("contains non-ASCII chars"),
            Error::Io(e) => e.fmt(f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl StdError for Error {}
