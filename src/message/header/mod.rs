//! Headers widely used in email messages

use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    ops::Deref,
};

pub use self::content_disposition::ContentDisposition;
pub use self::content_type::{ContentType, ContentTypeErr};
pub use self::date::Date;
pub use self::{content::*, mailbox::*, special::*, textual::*};
use crate::BoxError;

mod content;
mod content_disposition;
mod content_type;
mod date;
mod mailbox;
mod special;
mod textual;

pub trait Header: Clone {
    fn name() -> HeaderName;

    fn parse(s: &str) -> Result<Self, BoxError>;

    fn display(&self) -> String;
}

#[derive(Debug, Clone, Default)]
pub struct Headers {
    headers: Vec<(HeaderName, String)>,
}

impl Headers {
    #[inline]
    pub const fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            headers: Vec::with_capacity(capacity),
        }
    }

    pub fn get<H: Header>(&self) -> Option<H> {
        self.get_raw(&H::name()).and_then(|raw| H::parse(raw).ok())
    }

    pub fn set<H: Header>(&mut self, header: H) {
        self.set_raw(H::name(), header.display());
    }

    pub fn remove<H: Header>(&mut self) -> Option<H> {
        self.remove_raw(&H::name())
            .and_then(|(_name, raw)| H::parse(&raw).ok())
    }

    #[inline]
    pub fn clear(&mut self) {
        self.headers.clear();
    }

    pub fn get_raw(&self, name: &str) -> Option<&str> {
        self.find_header(name).map(|(_name, value)| value)
    }

    pub fn get_raw_mut(&mut self, name: &str) -> Option<&mut String> {
        self.find_header_mut(name).map(|(_name, value)| value)
    }

    pub fn insert_raw(&mut self, name: HeaderName, value: String) {
        match self.find_header_mut(&name) {
            Some((_name, prev_value)) => {
                prev_value.push_str(", ");
                prev_value.push_str(&value);
            }
            None => self.headers.push((name, value)),
        }
    }

    pub fn set_raw(&mut self, name: HeaderName, value: String) {
        match self.find_header_mut(&name) {
            Some((_, current_value)) => {
                *current_value = value;
            }
            None => {
                self.headers.push((name, value));
            }
        }
    }

    pub fn remove_raw(&mut self, name: &str) -> Option<(HeaderName, String)> {
        self.find_header_index(name).map(|i| self.headers.remove(i))
    }

    fn find_header(&self, name: &str) -> Option<(&HeaderName, &str)> {
        self.headers
            .iter()
            .find(|&(name_, _value)| name.eq_ignore_ascii_case(name_))
            .map(|t| (&t.0, t.1.as_str()))
    }

    fn find_header_mut(&mut self, name: &str) -> Option<(&HeaderName, &mut String)> {
        self.headers
            .iter_mut()
            .find(|(name_, _value)| name.eq_ignore_ascii_case(name_))
            .map(|t| (&t.0, &mut t.1))
    }

    fn find_header_index(&self, name: &str) -> Option<usize> {
        self.headers
            .iter()
            .enumerate()
            .find(|&(_i, (name_, _value))| name.eq_ignore_ascii_case(name_))
            .map(|(i, _)| i)
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, value) in &self.headers {
            Display::fmt(name, f)?;
            f.write_str(": ")?;
            Display::fmt(value, f)?;
            f.write_str("\r\n")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct HeaderName(Cow<'static, str>);

impl HeaderName {
    pub fn new_from_ascii(ascii: String) -> Self {
        assert!(ascii.is_ascii());
        assert!(!ascii.is_empty() && ascii.len() <= 76);
        assert!(ascii.trim().len() == ascii.len());
        assert!(!ascii.contains(':'));
        Self(Cow::Owned(ascii))
    }

    pub const fn new_from_ascii_str(ascii: &'static str) -> Self {
        let make_panic = [(); 1];

        let bytes = ascii.as_bytes();
        // the following line panics if ascii is longer than 76 characters
        let _ = make_panic[(bytes.is_empty() || bytes.len() > 76) as usize];
        let mut i = 0;
        while i < bytes.len() {
            let is_ascii = bytes[i].is_ascii();
            // the following line panics if the character isn't ascii
            let _ = make_panic[!is_ascii as usize];
            let is_unacceptable_char = bytes[i] == b' ' || bytes[i] == b':';
            // the following line panics if the character isn't acceptable in an header name
            let _ = make_panic[is_unacceptable_char as usize];
            i += 1;
        }

        Self(Cow::Borrowed(ascii))
    }
}

impl Display for HeaderName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self)
    }
}

impl Deref for HeaderName {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for HeaderName {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        let s: &str = self.as_ref();
        s.as_bytes()
    }
}

impl AsRef<str> for HeaderName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<HeaderName> for HeaderName {
    fn eq(&self, other: &HeaderName) -> bool {
        let s1: &str = self.as_ref();
        let s2: &str = other.as_ref();
        s1 == s2
    }
}

impl PartialEq<&str> for HeaderName {
    fn eq(&self, other: &&str) -> bool {
        let s: &str = self.as_ref();
        s == *other
    }
}

impl PartialEq<HeaderName> for &str {
    fn eq(&self, other: &HeaderName) -> bool {
        let s: &str = other.as_ref();
        *self == s
    }
}
