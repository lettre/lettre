use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

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
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        fmt.write_str(&match self {
            Error::MissingFrom => "missing source address, invalid envelope".to_string(),
            Error::MissingTo => "missing destination address, invalid envelope".to_string(),
            Error::TooManyFrom => "there can only be one source address".to_string(),
            Error::EmailMissingAt => "missing @ in email address".to_string(),
            Error::EmailMissingLocalPart => "missing local part in email address".to_string(),
            Error::EmailMissingDomain => "missing domain in email address".to_string(),
            Error::CannotParseFilename => "could not parse attachment filename".to_string(),
            Error::Io(e) => e.to_string(),
        })
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}
