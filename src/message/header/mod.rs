//! Headers widely used in email messages
// https://tools.ietf.org/html/rfc5322#section-2.2

use std::{
    borrow::Cow,
    fmt::{self, Display, Write},
    ops::{Deref, DerefMut},
};

mod content;
mod mailbox;
mod special;
mod textual;

pub use self::{content::*, mailbox::*, special::*, textual::*};

pub trait Header: Clone {
    fn name() -> HeaderName;

    fn parse_value(s: &str) -> Self;

    fn display(&self) -> String;
}

#[derive(Debug, Clone, Default)]
pub struct Headers {
    headers: Vec<(HeaderName, String)>,
}

#[derive(Debug, Clone)]
pub struct HeaderName(Cow<'static, str>);

pub struct RawHeaderHandle<'a, H: Header> {
    value: &'a mut String,
    header: H,
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
        self.get_raw(&H::name()).map(|raw| H::parse_value(raw))
    }

    pub fn get_mut<H: Header>(&mut self) -> Option<RawHeaderHandle<'_, H>> {
        let value = self.get_raw_mut(&H::name())?;
        let header = Header::parse_value(&value);

        Some(RawHeaderHandle { value, header })
    }

    pub fn set<H: Header>(&mut self, header: H) {
        self.set_raw(H::name(), header.display());
    }

    pub fn remove<H: Header>(&mut self) -> Option<H> {
        self.remove_raw(&H::name())
            .map(|(_name, raw)| H::parse_value(&raw))
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

    #[inline]
    pub fn clear(&mut self) {
        self.headers.clear();
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
            let encoder = HeaderEncoder::new(f, &name, &value)?;
            encoder.format(f)?;

            /*
            Display::fmt(name, f)?;
            f.write_str(": ")?;
            Display::fmt(value, f)?;
            f.write_str("\r\n")?;
            */

            f.write_str("\r\n")?;
        }

        Ok(())
    }
}

fn allowed_str(s: &str) -> bool {
    s.chars().all(allowed_char)
}

fn allowed_char(c: char) -> bool {
    c >= 1 as char && c <= 9 as char
        || c == 11 as char
        || c == 12 as char
        || c >= 14 as char && c <= 127 as char
}

const MAX_LINE_LEN: usize = 76;

struct HeaderEncoder<'a> {
    words_iter: Option<WordsPlusFillIterator<'a>>,

    line_len: usize,
    encode_buf: String,
}

impl<'a> HeaderEncoder<'a> {
    fn new(f: &mut fmt::Formatter<'_>, name: &str, value: &'a str) -> Result<Self, fmt::Error> {
        f.write_str(name)?;
        f.write_str(": ")?;

        Ok(Self {
            words_iter: Some(WordsPlusFillIterator { s: value }),

            line_len: name.len() + 2,
            encode_buf: String::new(),
        })
    }

    fn format(mut self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn would_fit_new_line(len: usize) -> bool {
            len < (MAX_LINE_LEN - 1)
        }

        fn base64_len(len: usize) -> usize {
            "=?utf-8?b?".len() + (len * 4 / 3 + 4) + "?=".len()
        }

        fn available_len_to_max_encode_len(len: usize) -> usize {
            len.saturating_sub("=?utf-8?b?".len() + (len * 3 / 4 + 4) + "?=".len())
        }

        let iter = self.words_iter.take().unwrap();
        for next_word in iter {
            let allowed = allowed_str(next_word);

            if allowed {
                // the next word is allowed, but we may have accumulated some words to encode
                self.flush_encode_buf(f, true)?;

                if next_word.len() > self.remaining_line_len() {
                    // not enough space left on this line to encode word

                    if self.something_written_to_this_line() && would_fit_new_line(next_word.len())
                    {
                        // word doesn't fit this line, but something had already been written to it,
                        // and word would fit the next line, so go to a new line
                        // so go to new line
                        self.new_line(f)?;
                    } else {
                        // word neither fits this line and the next one, cut it
                        // in the middle and make it fit

                        let mut next_word = next_word;

                        while !next_word.is_empty() {
                            if self.remaining_line_len() == 0 {
                                self.new_line(f)?;
                            }

                            let len = self.remaining_line_len().min(next_word.len());
                            let first_part = &next_word[..len];
                            next_word = &next_word[len..];

                            f.write_str(first_part)?;
                            self.line_len += first_part.len();
                        }

                        continue;
                    }
                }

                // word fits, write it!
                f.write_str(next_word)?;
                self.line_len += next_word.len();
            } else {
                if self.remaining_line_len() >= base64_len(self.encode_buf.len() + next_word.len())
                {
                    // next_word fits
                    self.encode_buf.push_str(next_word);
                    continue;
                }

                // next_word doesn't fit this line

                if would_fit_new_line(base64_len(next_word.len())) {
                    // ...but it would fit the next one

                    self.flush_encode_buf(f, false)?;
                    self.new_line(f)?;

                    self.encode_buf.push_str(next_word);
                    continue;
                }

                // ...and also wouldn't fit the next one.
                // chop it up into pieces

                let mut next_word = next_word;

                while !next_word.is_empty() {
                    if self.remaining_line_len() <= base64_len(1) {
                        self.flush_encode_buf(f, false)?;
                        self.new_line(f)?;
                    }

                    // FIXME: don't cut the string on a char boundary

                    let len = available_len_to_max_encode_len(self.remaining_line_len())
                        .min(next_word.len());
                    let first_part = &next_word[..len];
                    next_word = &next_word[len..];

                    self.encode_buf.push_str(first_part);
                }
            }
        }

        self.flush_encode_buf(f, false)?;

        Ok(())
    }

    fn remaining_line_len(&self) -> usize {
        MAX_LINE_LEN - self.line_len
    }

    fn something_written_to_this_line(&self) -> bool {
        self.line_len > 1
    }

    fn flush_encode_buf(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        switching_to_allowed: bool,
    ) -> fmt::Result {
        if !self.encode_buf.is_empty() {
            let mut write_after = None;

            if switching_to_allowed {
                let last_char = self.encode_buf.pop().unwrap();
                if is_space_like(last_char) {
                    write_after = Some(last_char);
                } else {
                    self.encode_buf.push(last_char);
                }
            }

            f.write_str("=?utf-8?b?")?;
            let encoded = base64::display::Base64Display::with_config(
                self.encode_buf.as_bytes(),
                base64::STANDARD,
            );
            Display::fmt(&encoded, f)?;
            f.write_str("?=")?;

            self.line_len += "=?utf-8?b?".len();
            self.line_len += self.encode_buf.len() * 4 / 3 + 4;
            self.line_len += "?=".len();

            if let Some(write_after) = write_after {
                f.write_char(write_after)?;
                self.line_len += 1;
            }

            self.encode_buf.clear();
        }

        Ok(())
    }

    fn new_line(&mut self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\r\n ")?;
        self.line_len = 1;

        Ok(())
    }
}

struct WordsPlusFillIterator<'a> {
    s: &'a str,
}

impl<'a> Iterator for WordsPlusFillIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.s.is_empty() {
            return None;
        }

        let next_word = self
            .s
            .char_indices()
            .skip(1)
            .skip_while(|&(_i, c)| !is_space_like(c))
            .find(|&(_i, c)| !is_space_like(c))
            .map(|(i, _)| i);

        let word = &self.s[..next_word.unwrap_or_else(|| self.s.len())];
        self.s = &self.s[word.len()..];
        Some(word)
    }
}

fn is_space_like(c: char) -> bool {
    c == ',' || c == ' '
}

impl HeaderName {
    pub fn new_from_ascii(ascii: String) -> Self {
        assert!(ascii.is_ascii());
        assert!(!ascii.is_empty() && ascii.len() <= 76);
        assert!(ascii.trim().len() == ascii.len());
        assert!(!ascii.contains(':'));
        Self(Cow::Owned(ascii))
    }

    pub const fn new_from_ascii_static(ascii: &'static str) -> Self {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl<'a, H: Header> Deref for RawHeaderHandle<'a, H> {
    type Target = H;
    fn deref(&self) -> &Self::Target {
        &self.header
    }
}

impl<'a, H: Header> DerefMut for RawHeaderHandle<'a, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.header
    }
}

impl<'a, H: Header> Drop for RawHeaderHandle<'a, H> {
    fn drop(&mut self) {
        *self.value = self.header.display();
    }
}

#[cfg(test)]
mod tests {
    use super::HeaderName;

    #[test]
    fn valid_headername() {
        assert_eq!(HeaderName::new_from_ascii(String::from("From")), "From");
        assert_eq!(HeaderName::new_from_ascii(String::from("X-Duck")), "X-Duck");
    }

    #[should_panic]
    #[test]
    fn invalid_headername_1() {
        HeaderName::new_from_ascii(String::from("From:"));
    }

    #[should_panic]
    #[test]
    fn invalid_headername_2() {
        HeaderName::new_from_ascii(String::from("Date "));
    }

    #[should_panic]
    #[test]
    fn invalid_headername_3() {
        HeaderName::new_from_ascii(String::from("✉️"));
    }

    #[test]
    fn valid_headername_static() {
        assert_eq!(HeaderName::new_from_ascii_static("From"), "From");
        assert_eq!(HeaderName::new_from_ascii_static("X-Duck"), "X-Duck");
    }

    #[should_panic]
    #[test]
    fn invalid_headername_static_1() {
        HeaderName::new_from_ascii_static("From:");
    }

    #[should_panic]
    #[test]
    fn invalid_headername_static_2() {
        HeaderName::new_from_ascii_static("Date ");
    }

    #[should_panic]
    #[test]
    fn invalid_headername_static_3() {
        HeaderName::new_from_ascii_static("✉️");
    }
}
