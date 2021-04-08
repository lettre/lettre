use std::{fmt::Result as FmtResult, str::from_utf8, time::SystemTime};

use httpdate::HttpDate;
use hyperx::{
    header::{Formatter as HeaderFormatter, Header, RawLike},
    Error as HeaderError, Result as HyperResult,
};

/// Message `Date` header
///
/// Defined in [RFC2822](https://tools.ietf.org/html/rfc2822#section-3.3)
#[derive(Debug, Clone, Copy, PartialEq)]
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
        Self::new(SystemTime::now())
    }
}

impl Header for Date {
    fn header_name() -> &'static str {
        "Date"
    }

    // FIXME HeaderError->HeaderError, same for result
    fn parse_header<'a, T>(raw: &'a T) -> HyperResult<Self>
    where
        T: RawLike<'a>,
        Self: Sized,
    {
        raw.one()
            .ok_or(HeaderError::Header)
            .and_then(|r| from_utf8(r).map_err(|_| HeaderError::Header))
            .and_then(|s| {
                let mut s = String::from(s);
                if s.ends_with(" -0000") {
                    // The httpdate crate expects the `Date` to end in ` GMT`, but email
                    // uses `-0000`, so we crudely fix this issue here.

                    s.truncate(s.len() - "-0000".len());
                    s.push_str("GMT");
                }

                s.parse::<HttpDate>()
                    .map(Self)
                    .map_err(|_| HeaderError::Header)
            })
    }

    fn fmt_header(&self, f: &mut HeaderFormatter<'_, '_>) -> FmtResult {
        let mut s = self.0.to_string();
        if s.ends_with(" GMT") {
            // The httpdate crate always appends ` GMT` to the end of the string,
            // but this is considered an obsolete date format for email
            // https://tools.ietf.org/html/rfc2822#appendix-A.6.2,
            // so we replace `GMT` with `-0000`
            s.truncate(s.len() - "GMT".len());
            s.push_str("-0000");
        }

        f.fmt_line(&s)
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
    use hyperx::header::Headers;
    use std::time::{Duration, SystemTime};

    use super::Date;

    #[test]
    fn format_date() {
        let mut headers = Headers::new();

        // Tue, 15 Nov 1994 08:12:31 GMT
        headers.set(Date::from(
            SystemTime::UNIX_EPOCH + Duration::from_secs(784887151),
        ));

        assert_eq!(
            format!("{}", headers),
            "Date: Tue, 15 Nov 1994 08:12:31 -0000\r\n"
        );

        // Tue, 15 Nov 1994 08:12:32 GMT
        headers.set(Date::from(
            SystemTime::UNIX_EPOCH + Duration::from_secs(784887152),
        ));

        assert_eq!(
            format!("{}", headers),
            "Date: Tue, 15 Nov 1994 08:12:32 -0000\r\n"
        );
    }

    #[test]
    fn parse_date() {
        let mut headers = Headers::new();

        headers.set_raw("Date", "Tue, 15 Nov 1994 08:12:31 -0000");

        assert_eq!(
            headers.get::<Date>(),
            Some(&Date::from(
                SystemTime::UNIX_EPOCH + Duration::from_secs(784887151),
            ))
        );

        headers.set_raw("Date", "Tue, 15 Nov 1994 08:12:32 -0000");

        assert_eq!(
            headers.get::<Date>(),
            Some(&Date::from(
                SystemTime::UNIX_EPOCH + Duration::from_secs(784887152),
            ))
        );
    }
}
