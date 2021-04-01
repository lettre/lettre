use httpdate::HttpDate;

use super::{Header, HeaderName};
use std::{fmt::Result as FmtResult, str::from_utf8, time::SystemTime};

/// https://tools.ietf.org/html/rfc2822
#[allow(missing_copy_implementations)]
#[derive(Clone)]
pub struct Date(HttpDate);

impl Date {
    pub fn now() -> Self {
        Self::from(SystemTime::now())
    }
}

impl Header for Date {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_static("Date")
    }

    fn parse_value(s: &str) -> Self {
        let mut s = String::from(s);
        let s = if s.ends_with(" -0000") {
            s.truncate(s.len() - "-0000".len());
            // UTC
            s.push_str("GMT");
            s
        } else {
            // UNEXPECTED
            s
        };

        println!("{}", s);
        Self(s.parse().unwrap())
    }

    fn display(&self) -> String {
        let mut s = self.0.to_string();
        if s.ends_with(" GMT") {
            s.truncate(s.len() - "GMT".len());
            // UTC
            s.push_str("-0000");
            s
        } else {
            // UNEXPECTED
            s
        }
    }
}

impl From<SystemTime> for Date {
    fn from(st: SystemTime) -> Self {
        Self(HttpDate::from(st))
    }
}

impl From<Date> for SystemTime {
    #[inline]
    fn from(this: Date) -> SystemTime {
        this.0.into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Message format version, defined in [RFC2045](https://tools.ietf.org/html/rfc2045#section-4)
pub struct MimeVersion {
    major: u8,
    minor: u8,
}

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

impl Default for MimeVersion {
    fn default() -> Self {
        MIME_VERSION_1_0
    }
}

impl Header for MimeVersion {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_static("MIME-Version")
    }

    fn parse_value(s: &str) -> Self {
        let mut split = s.split('.');

        let major = split.next().unwrap();
        let minor = split.next().unwrap();
        let major = major.parse().unwrap();
        let minor = minor.parse().unwrap();
        MimeVersion::new(major, minor)
    }

    fn display(&self) -> String {
        format!("{}.{}", self.major, self.minor)
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

        assert_eq!(format!("{}", headers), "MIME-Version: 1.0\r\n");

        headers.set(MimeVersion::new(0, 1));

        assert_eq!(format!("{}", headers), "MIME-Version: 0.1\r\n");
    }

    #[test]
    fn parse_mime_version() {
        let mut headers = Headers::new();

        headers.set_raw(
            HeaderName::new_from_ascii_static("MIME-Version"),
            "1.0".into(),
        );

        assert_eq!(headers.get::<MimeVersion>(), Some(MIME_VERSION_1_0));

        headers.set_raw(
            HeaderName::new_from_ascii_static("MIME-Version"),
            "0.1".into(),
        );

        assert_eq!(headers.get::<MimeVersion>(), Some(MimeVersion::new(0, 1)));
    }
}
