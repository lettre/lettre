use std::time::SystemTime;

use httpdate::HttpDate;

use super::{Header, HeaderName, HeaderValue};
use crate::BoxError;

/// Message `Date` header
///
/// Defined in [RFC2822](https://tools.ietf.org/html/rfc2822#section-3.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date(HttpDate);

impl Date {
    /// Build a `Date` from [`SystemTime`]
    pub fn new(st: SystemTime) -> Self {
        Self(st.into())
    }

    /// Get the current date
    ///
    /// Shortcut for `Date::new(SystemTime::now())`
    pub fn now() -> Self {
        #[cfg(not(feature = "web"))]
        return Self::new(SystemTime::now());
        #[cfg(feature = "web")]
        return Self::new(crate::message::to_std_systemtime(
            web_time::SystemTime::now(),
        ));
    }
}

impl Header for Date {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("Date")
    }

    fn parse(s: &str) -> Result<Self, BoxError> {
        let mut s = String::from(s);
        if s.ends_with("+0000") {
            // The httpdate crate expects the `Date` to end in ` GMT`, but email
            // uses `+0000` to indicate UTC, so we crudely fix this issue here.

            s.truncate(s.len() - "+0000".len());
            s.push_str("GMT");
        }

        Ok(Self(s.parse::<HttpDate>()?))
    }

    fn display(&self) -> HeaderValue {
        let mut val = self.0.to_string();
        if val.ends_with(" GMT") {
            // The httpdate crate always appends ` GMT` to the end of the string,
            // but this is considered an obsolete date format for email
            // https://tools.ietf.org/html/rfc2822#appendix-A.6.2,
            // so we replace `GMT` with `+0000`
            val.truncate(val.len() - "GMT".len());
            val.push_str("+0000");
        }

        HeaderValue::dangerous_new_pre_encoded(Self::name(), val.clone(), val)
    }
}

impl From<SystemTime> for Date {
    fn from(st: SystemTime) -> Self {
        Self::new(st)
    }
}

impl From<Date> for SystemTime {
    fn from(this: Date) -> SystemTime {
        this.0.into()
    }
}

#[cfg(test)]
mod test {
    use std::time::{Duration, SystemTime};

    use pretty_assertions::assert_eq;

    use super::Date;
    use crate::message::header::{HeaderName, HeaderValue, Headers};

    #[test]
    fn format_date() {
        let mut headers = Headers::new();

        // Tue, 15 Nov 1994 08:12:31 GMT
        headers.set(Date::from(
            SystemTime::UNIX_EPOCH + Duration::from_secs(784887151),
        ));

        assert_eq!(
            headers.to_string(),
            "Date: Tue, 15 Nov 1994 08:12:31 +0000\r\n".to_owned()
        );

        // Tue, 15 Nov 1994 08:12:32 GMT
        headers.set(Date::from(
            SystemTime::UNIX_EPOCH + Duration::from_secs(784887152),
        ));

        assert_eq!(
            headers.to_string(),
            "Date: Tue, 15 Nov 1994 08:12:32 +0000\r\n"
        );
    }

    #[test]
    fn parse_date() {
        let mut headers = Headers::new();

        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("Date"),
            "Tue, 15 Nov 1994 08:12:31 +0000".to_owned(),
        ));

        assert_eq!(
            headers.get::<Date>(),
            Some(Date::from(
                SystemTime::UNIX_EPOCH + Duration::from_secs(784887151),
            ))
        );

        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("Date"),
            "Tue, 15 Nov 1994 08:12:32 +0000".to_owned(),
        ));

        assert_eq!(
            headers.get::<Date>(),
            Some(Date::from(
                SystemTime::UNIX_EPOCH + Duration::from_secs(784887152),
            ))
        );
    }
}
