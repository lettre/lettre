use std::{
    fmt,
    time::{Duration, SystemTime},
};

use httpdate::HttpDate;

use super::{Header, HeaderName, HeaderValue};
use crate::BoxError;

const NUMERIC_OFFSET_LEN: usize = 5;

/// UTC offset of a [`Date`], in minutes east of UTC
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Offset(i16);

impl Offset {
    /// UTC (`+0000`)
    pub const UTC: Self = Self(0);

    /// Build from minutes east of UTC
    ///
    /// Returns an error for magnitudes above 1439 (23:59).
    pub fn from_minutes(minutes: i16) -> Result<Self, BoxError> {
        if (-1439..=1439).contains(&minutes) {
            Ok(Self(minutes))
        } else {
            Err("offset out of range".into())
        }
    }

    /// Minutes east of UTC
    pub fn minutes(self) -> i16 {
        self.0
    }
}

impl fmt::Display for Offset {
    /// Formats the offset as a numeric `±HHMM` zone
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let minutes = self.0.unsigned_abs();
        let sign = if self.0 < 0 { '-' } else { '+' };
        write!(f, "{}{:02}{:02}", sign, minutes / 60, minutes % 60)
    }
}

/// Message `Date` header
///
/// Defined in [RFC2822](https://tools.ietf.org/html/rfc2822#section-3.3)
///
/// Carries the UTC offset at which the message was sent alongside the
/// wall-clock time, rendered as a numeric `±HHMM` zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    // Wall-clock as presented; httpdate renders it with a trailing ` GMT`
    wall: HttpDate,
    offset: Offset,
}

impl Date {
    /// Build a `Date` from [`SystemTime`], in UTC
    pub fn new(st: SystemTime) -> Self {
        Self {
            wall: st.into(),
            offset: Offset::UTC,
        }
    }

    /// Build a `Date` from [`SystemTime`] at a given UTC [`Offset`]
    ///
    /// The wall-clock time and weekday are derived from the instant shifted by
    /// `offset`; the calendar math is done by httpdate.
    pub fn new_with_offset(st: SystemTime, offset: Offset) -> Self {
        let secs = i64::from(offset.minutes()) * 60;
        Self {
            wall: HttpDate::from(shift(st, secs)),
            offset,
        }
    }

    /// Get the current date
    ///
    /// Shortcut for `Date::new(SystemTime::now())`
    pub fn now() -> Self {
        Self::new(crate::time::now())
    }

    /// Get the UTC offset of this date
    pub fn offset(self) -> Offset {
        self.offset
    }
}

/// Split off a trailing numeric zone (`[+-]DDMM`), returning the date without
/// the zone and the offset in minutes east of UTC
fn split_numeric_zone(s: &str) -> Option<(&str, Offset)> {
    let zone = s.get(s.len().checked_sub(NUMERIC_OFFSET_LEN)?..)?;
    let digits = zone.strip_prefix(['+', '-'])?.parse::<i16>().ok()?;
    let (hours, minutes) = (digits / 100, digits % 100);
    if digits < 0 || hours > 23 || minutes > 59 {
        return None;
    }
    let total = hours * 60 + minutes;
    let offset = Offset(if zone.starts_with('-') { -total } else { total });
    Some((&s[..s.len() - NUMERIC_OFFSET_LEN], offset))
}

/// Move `st` by `secs`, positive meaning later
fn shift(st: SystemTime, secs: i64) -> SystemTime {
    let dur = Duration::from_secs(secs.unsigned_abs());
    if secs >= 0 { st + dur } else { st - dur }
}

impl Header for Date {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("Date")
    }

    fn parse(s: &str) -> Result<Self, BoxError> {
        if s.ends_with("-0000") {
            // `-0000` means the local time is unknown, not UTC
            // https://tools.ietf.org/html/rfc2822#section-3.3
            return Err("-0000 (unknown local time) is not supported".into());
        }

        // The httpdate crate expects the date to end in ` GMT`, but email
        // dates carry a numeric UT offset, so swap the zone for ` GMT`
        if let Some((rest, offset)) = split_numeric_zone(s) {
            let mut wall_src = String::from(rest);
            wall_src.push_str("GMT");
            return Ok(Self {
                wall: wall_src.parse::<HttpDate>()?,
                offset,
            });
        }

        // Dates already ending in `GMT` (and asctime-style dates without a
        // zone) parse as-is
        Ok(Self {
            wall: s.parse::<HttpDate>()?,
            offset: Offset::UTC,
        })
    }

    fn display(&self) -> HeaderValue {
        let mut val = self.wall.to_string();
        if val.ends_with(" GMT") {
            // The httpdate crate always appends ` GMT` to the end of the
            // string, but this is considered an obsolete date format for email
            // https://tools.ietf.org/html/rfc2822#appendix-A.6.2,
            // so we replace `GMT` with the numeric `±HHMM` offset
            val.truncate(val.len() - "GMT".len());
            val.push_str(&self.offset.to_string());
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
        shift(this.wall.into(), -i64::from(this.offset.minutes()) * 60)
    }
}

#[cfg(test)]
mod test {
    use std::time::{Duration, SystemTime};

    use pretty_assertions::assert_eq;

    use super::{Date, Offset};
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

    #[test]
    fn format_date_with_offset() {
        let mut headers = Headers::new();

        // Tue, 15 Nov 1994 08:12:31 GMT
        headers.set(Date::new_with_offset(
            SystemTime::UNIX_EPOCH + Duration::from_secs(784887151),
            Offset::from_minutes(480).unwrap(),
        ));

        assert_eq!(
            headers.to_string(),
            "Date: Tue, 15 Nov 1994 16:12:31 +0800\r\n".to_owned()
        );
    }

    #[test]
    fn format_date_with_negative_offset_crossing_midnight() {
        let mut headers = Headers::new();

        // Tue, 15 Nov 1994 08:12:31 GMT
        headers.set(Date::new_with_offset(
            SystemTime::UNIX_EPOCH + Duration::from_secs(784887151),
            Offset::from_minutes(-600).unwrap(),
        ));

        assert_eq!(
            headers.to_string(),
            "Date: Mon, 14 Nov 1994 22:12:31 -1000\r\n".to_owned()
        );
    }

    #[test]
    fn parse_date_with_offset() {
        let mut headers = Headers::new();

        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("Date"),
            "Wed, 01 Jan 2025 00:00:00 +0800".to_owned(),
        ));

        // 1735660800 = 2024-12-31T16:00:00Z, +8h wall = Wed, 01 Jan 2025
        assert_eq!(
            headers.get::<Date>(),
            Some(Date::new_with_offset(
                SystemTime::UNIX_EPOCH + Duration::from_secs(1735660800),
                Offset::from_minutes(480).unwrap(),
            ))
        );

        // parse → display roundtrip is byte-exact
        headers.set(headers.get::<Date>().unwrap());
        assert_eq!(
            headers.to_string(),
            "Date: Wed, 01 Jan 2025 00:00:00 +0800\r\n".to_owned()
        );
    }

    #[test]
    fn parse_rejects_unknown_local_time() {
        let mut headers = Headers::new();

        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("Date"),
            "Tue, 15 Nov 1994 08:12:31 -0000".to_owned(),
        ));

        assert_eq!(headers.get::<Date>(), None);
    }

    #[test]
    fn system_time_roundtrip_with_offset() {
        let instant = SystemTime::UNIX_EPOCH + Duration::from_secs(1735660800);

        assert_eq!(
            SystemTime::from(Date::new_with_offset(
                instant,
                Offset::from_minutes(480).unwrap(),
            )),
            instant
        );
    }

    #[test]
    fn parse_non_ascii_date_does_not_panic() {
        let mut headers = Headers::new();

        headers.insert_raw(HeaderValue::new(
            HeaderName::new_from_ascii_str("Date"),
            "ééééé".to_owned(),
        ));

        assert_eq!(headers.get::<Date>(), None);
    }

    #[test]
    fn offset_from_minutes_range() {
        assert_eq!(Offset::from_minutes(1439).unwrap().minutes(), 1439);
        assert_eq!(Offset::from_minutes(-1439).unwrap().minutes(), -1439);
        assert!(Offset::from_minutes(1440).is_err());
        assert!(Offset::from_minutes(-1440).is_err());
    }
}
