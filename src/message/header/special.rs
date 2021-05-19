use crate::{
    message::header::{Header, HeaderName},
    BoxError,
};

/// Message format version, defined in [RFC2045](https://tools.ietf.org/html/rfc2045#section-4)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MimeVersion {
    major: u8,
    minor: u8,
}

/// MIME version 1.0
///
/// Should be used in all MIME messages.
pub const MIME_VERSION_1_0: MimeVersion = MimeVersion::new(1, 0);

impl MimeVersion {
    pub const fn new(major: u8, minor: u8) -> Self {
        MimeVersion { major, minor }
    }

    #[inline]
    pub const fn major(self) -> u8 {
        self.major
    }

    #[inline]
    pub const fn minor(self) -> u8 {
        self.minor
    }
}

impl Header for MimeVersion {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("MIME-Version")
    }

    fn parse(s: &str) -> Result<Self, BoxError> {
        let mut s = s.split('.');

        let major = s
            .next()
            .expect("The first call to next for a Split<char> always succeeds");
        let minor = s
            .next()
            .ok_or_else(|| String::from("MIME-Version header doesn't contain '.'"))?;
        let major = major.parse()?;
        let minor = minor.parse()?;
        Ok(MimeVersion::new(major, minor))
    }

    fn display(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

impl Default for MimeVersion {
    fn default() -> Self {
        MIME_VERSION_1_0
    }
}

#[cfg(test)]
mod test {
    use super::{MimeVersion, MIME_VERSION_1_0};
    use crate::message::header::{HeaderName, Headers};

    #[test]
    fn format_mime_version() {
        let mut headers = Headers::new();

        headers.set(MIME_VERSION_1_0);

        assert_eq!(headers.to_string(), "MIME-Version: 1.0\r\n");

        headers.set(MimeVersion::new(0, 1));

        assert_eq!(headers.to_string(), "MIME-Version: 0.1\r\n");
    }

    #[test]
    fn parse_mime_version() {
        let mut headers = Headers::new();

        headers.insert_raw(
            HeaderName::new_from_ascii_str("MIME-Version"),
            "1.0".to_string(),
        );

        assert_eq!(headers.get::<MimeVersion>(), Some(MIME_VERSION_1_0));

        headers.insert_raw(
            HeaderName::new_from_ascii_str("MIME-Version"),
            "0.1".to_string(),
        );

        assert_eq!(headers.get::<MimeVersion>(), Some(MimeVersion::new(0, 1)));
    }
}
