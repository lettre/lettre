use failure;

/// Error type for email content
#[derive(Fail, Debug, Clone, Copy)]
pub enum Error {
    /// Missing from in envelope
    #[fail(display = "missing source address, invalid envelope")]
    MissingFrom,
    /// Missing to in envelope
    #[fail(display = "missing destination address, invalid envelope")]
    MissingTo,
    /// Invalid email
    #[fail(display = "invalid email address")]
    InvalidEmailAddress,
}

/// Email result type
pub type EmailResult<T> = Result<T, failure::Error>;
