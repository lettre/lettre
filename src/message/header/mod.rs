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
            HeaderValueEncoder::encode(&name, &value, f)?;
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

const ENCODING_START_PREFIX: &str = "=?utf-8?b?";
const ENCODING_END_SUFFIX: &str = "?=";
const MAX_LINE_LEN: usize = 76;

/// [RFC 1522](https://tools.ietf.org/html/rfc1522) header value encoder
struct HeaderValueEncoder {
    line_len: usize,
    encode_buf: String,
}

impl HeaderValueEncoder {
    fn encode(name: &str, value: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (words_iter, encoder) = Self::new(name, value);
        encoder.format(words_iter, f)
    }

    fn new<'a>(name: &str, value: &'a str) -> (WordsPlusFillIterator<'a>, Self) {
        (
            WordsPlusFillIterator { s: value },
            Self {
                line_len: name.len() + ": ".len(),
                encode_buf: String::new(),
            },
        )
    }

    fn format(
        mut self,
        words_iter: WordsPlusFillIterator<'_>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        /// Estimate if an encoded string of `len` would fix in an empty line
        fn would_fit_new_line(len: usize) -> bool {
            len < (MAX_LINE_LEN - " ".len())
        }

        /// Estimate how long a string of `len` would be after base64 encoding plus
        /// adding the encoding prefix and suffix to it
        fn base64_len(len: usize) -> usize {
            ENCODING_START_PREFIX.len() + (len * 4 / 3 + 4) + ENCODING_END_SUFFIX.len()
        }

        /// Estimate how many more bytes we can fit in the current line
        fn available_len_to_max_encode_len(len: usize) -> usize {
            len.saturating_sub(
                ENCODING_START_PREFIX.len() + (len * 3 / 4 + 4) + ENCODING_END_SUFFIX.len(),
            )
        }

        for next_word in words_iter {
            let allowed = allowed_str(next_word);

            if allowed {
                // This word only contains allowed characters

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
                // This word contains unallowed characters

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

    /// Returns the number of bytes left for the current line
    fn remaining_line_len(&self) -> usize {
        MAX_LINE_LEN - self.line_len
    }

    /// Returns true if something has been written to the current line
    fn something_written_to_this_line(&self) -> bool {
        self.line_len > 1
    }

    fn flush_encode_buf(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        switching_to_allowed: bool,
    ) -> fmt::Result {
        use std::fmt::Write;

        if self.encode_buf.is_empty() {
            // nothing to encode
            return Ok(());
        }

        let mut write_after = None;

        if switching_to_allowed {
            // If the next word only contains allowed characters, and the string to encode
            // ends with a space, take the space out of the part to encode

            let last_char = self.encode_buf.pop().expect("self.encode_buf isn't empty");
            if is_space_like(last_char) {
                write_after = Some(last_char);
            } else {
                self.encode_buf.push(last_char);
            }
        }

        f.write_str(ENCODING_START_PREFIX)?;
        let encoded = base64::display::Base64Display::with_config(
            self.encode_buf.as_bytes(),
            base64::STANDARD,
        );
        Display::fmt(&encoded, f)?;
        f.write_str(ENCODING_END_SUFFIX)?;

        self.line_len += ENCODING_START_PREFIX.len();
        self.line_len += self.encode_buf.len() * 4 / 3 + 4;
        self.line_len += ENCODING_END_SUFFIX.len();

        if let Some(write_after) = write_after {
            f.write_char(write_after)?;
            self.line_len += 1;
        }

        self.encode_buf.clear();
        Ok(())
    }

    fn new_line(&mut self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\r\n ")?;
        self.line_len = 1;

        Ok(())
    }
}

/// Iterator yielding a string split space by space, but including all space
/// characters between it and the next word
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

const fn is_space_like(c: char) -> bool {
    c == ',' || c == ' '
}

fn allowed_str(s: &str) -> bool {
    s.chars().all(allowed_char)
}

const fn allowed_char(c: char) -> bool {
    c >= 1 as char && c <= 9 as char
        || c == 11 as char
        || c == 12 as char
        || c >= 14 as char && c <= 127 as char
}

#[cfg(test)]
mod tests {
    use super::{HeaderName, Headers};

    // names taken randomly from https://it.wikipedia.org/wiki/Pinco_Pallino

    #[test]
    fn format_ascii() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("To"),
            "John Doe <example@example.com>, Jean Dupont <jean@example.com>".to_string(),
        );

        assert_eq!(
            headers.to_string(),
            "To: John Doe <example@example.com>, Jean Dupont <jean@example.com>\r\n"
        );
    }

    #[test]
    fn format_ascii_with_folding() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("To"),
            "Ascii <example@example.com>, John Doe <johndoe@example.com, John Smith <johnsmith@example.com>, Pinco Pallino <pincopallino@example.com>, Jemand <jemand@example.com>, Jean Dupont <jean@example.com>".to_string(),
        );

        assert_eq!(
            headers.to_string(),
            concat!(
                "To: Ascii <example@example.com>, John Doe <johndoe@example.com, John Smith \r\n",
                " <johnsmith@example.com>, Pinco Pallino <pincopallino@example.com>, Jemand \r\n",
                " <jemand@example.com>, Jean Dupont <jean@example.com>\r\n"
            )
        );
    }

    #[test]
    fn format_ascii_with_folding_long_line() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("Subject"),
            "Hello! This is lettre, and this IsAVeryLongLineDoYouKnowWhatsGoingToHappenIGuessWeAreGoingToFindOut. Ok I guess that's it!".to_string()
        );

        assert_eq!(
            headers.to_string(),
            concat!(
                "Subject: Hello! This is lettre, and this \r\n ",
                "IsAVeryLongLineDoYouKnowWhatsGoingToHappenIGuessWeAreGoingToFindOut. Ok I \r\n",
                " guess that's it!\r\n"
            )
        );
    }

    #[test]
    fn format_ascii_with_folding_very_long_line() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("Subject"),
            "Hello! IGuessTheLastLineWasntLongEnoughSoLetsTryAgainShallWeWhatDoYouThinkItsGoingToHappenIGuessWereAboutToFindOut! I don't know".to_string()
        );

        assert_eq!(
            headers.to_string(),
            concat!(
                "Subject: Hello! IGuessTheLastLineWasntLongEnoughSoLetsTryAgainShallWeWhatDoY\r\n",
                " ouThinkItsGoingToHappenIGuessWereAboutToFindOut! I don't know\r\n",
            )
        );
    }

    #[test]
    fn format_ascii_with_folding_giant_word() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("Subject"),
            "1abcdefghijklmnopqrstuvwxyz2abcdefghijklmnopqrstuvwxyz3abcdefghijklmnopqrstuvwxyz4abcdefghijklmnopqrstuvwxyz5abcdefghijklmnopqrstuvwxyz6abcdefghijklmnopqrstuvwxyz".to_string()
        );

        assert_eq!(
            headers.to_string(),
            concat!(
                "Subject: 1abcdefghijklmnopqrstuvwxyz2abcdefghijklmnopqrstuvwxyz3abcdefghijkl\r\n",
                " mnopqrstuvwxyz4abcdefghijklmnopqrstuvwxyz5abcdefghijklmnopqrstuvwxyz6abcdef\r\n",
                " ghijklmnopqrstuvwxyz\r\n",
            )
        );
    }

    #[test]
    fn format_special() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("To"),
            "Seán <sean@example.com>".to_string(),
        );

        assert_eq!(
            headers.to_string(),
            "To: =?utf-8?b?U2XDoW4=?= <sean@example.com>\r\n"
        );
    }

    #[test]
    fn format_special_emoji() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("To"),
            "🌎 <world@example.com>".to_string(),
        );

        assert_eq!(
            headers.to_string(),
            "To: =?utf-8?b?8J+Mjg==?= <world@example.com>\r\n"
        );
    }

    #[test]
    fn format_special_with_folding() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("To"),
            "🌍 <world@example.com>, 🦆 Everywhere <ducks@example.com>, Иванов Иван Иванович <ivanov@example.com>, Jānis Bērziņš <janis@example.com>, Seán Ó Rudaí <sean@example.com>".to_string(),
        );

        assert_eq!(
            headers.to_string(),
            concat!(
                "To: =?utf-8?b?8J+MjQ==?= <world@example.com>, =?utf-8?b?8J+mhg==?= \r\n",
                " Everywhere <ducks@example.com>, =?utf-8?b?0JjQstCw0L3QvtCyIA==?=\r\n",
                " =?utf-8?b?0JjQstCw0L0g0JjQstCw0L3QvtCy0LjRhw==?= <ivanov@example.com>, \r\n",
                " =?utf-8?b?SsSBbmlzIELEk3J6acWGxaE=?= <janis@example.com>, \r\n",
                " =?utf-8?b?U2XDoW4gw5MgUnVkYcOt?= <sean@example.com>\r\n"
            )
        );
    }

    #[test]
    fn format_bad_stuff() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("Subject"),
            "Hello! \r\n This is \" bad \0. 👋".to_string(),
        );

        assert_eq!(
            headers.to_string(),
            "Subject: Hello! =?utf-8?b?DQo=?= This is \" bad =?utf-8?b?AC4g8J+Riw==?=\r\n"
        );
    }

    #[test]
    fn format_everything() {
        let mut headers = Headers::new();
        headers.insert_raw(
            HeaderName::new_from_ascii_str("Subject"),
            "Hello! This is lettre, and this IsAVeryLongLineDoYouKnowWhatsGoingToHappenIGuessWeAreGoingToFindOut. Ok I guess that's it!".to_string()
        );
        headers.insert_raw(
            HeaderName::new_from_ascii_str("To"),
            "🌍 <world@example.com>, 🦆 Everywhere <ducks@example.com>, Иванов Иван Иванович <ivanov@example.com>, Jānis Bērziņš <janis@example.com>, Seán Ó Rudaí <sean@example.com>".to_string(),
        );
        headers.insert_raw(
            HeaderName::new_from_ascii_str("From"),
            "Someone <somewhere@example.com>".to_string(),
        );
        headers.insert_raw(
            HeaderName::new_from_ascii_str("Content-Transfer-Encoding"),
            "quoted-printable".to_string(),
        );

        assert_eq!(
            headers.to_string(),
            concat!(
                "Subject: Hello! This is lettre, and this \r\n",
                " IsAVeryLongLineDoYouKnowWhatsGoingToHappenIGuessWeAreGoingToFindOut. Ok I \r\n",
                " guess that's it!\r\n",
                "To: =?utf-8?b?8J+MjQ==?= <world@example.com>, =?utf-8?b?8J+mhg==?= \r\n",
                " Everywhere <ducks@example.com>, =?utf-8?b?0JjQstCw0L3QvtCyIA==?=\r\n",
                " =?utf-8?b?0JjQstCw0L0g0JjQstCw0L3QvtCy0LjRhw==?= <ivanov@example.com>, \r\n",
                " =?utf-8?b?SsSBbmlzIELEk3J6acWGxaE=?= <janis@example.com>, \r\n",
                " =?utf-8?b?U2XDoW4gw5MgUnVkYcOt?= <sean@example.com>\r\n",
                "From: Someone <somewhere@example.com>\r\n",
                "Content-Transfer-Encoding: quoted-printable\r\n",
            )
        );
    }
}
