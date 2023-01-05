use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Result as FmtResult, Write},
    mem,
    slice::Iter,
    str::FromStr,
};

use email_encoding::headers::EmailWriter;

use crate::address::{Address, AddressError};

/// Represents an email address with an optional name for the sender/recipient.
///
/// This type contains email address and the sender/recipient name (_Some Name \<user@domain.tld\>_ or _withoutname@domain.tld_).
///
/// **NOTE**: Enable feature "serde" to be able serialize/deserialize it using [serde](https://serde.rs/).
///
/// # Examples
///
/// You can create a `Mailbox` from a string and an [`Address`]:
///
/// ```
/// # use lettre::{Address, message::Mailbox};
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let address = Address::new("example", "email.com")?;
/// let mailbox = Mailbox::new(None, address);
/// # Ok(())
/// # }
/// ```
///
/// You can also create one from a string literal:
///
/// ```
/// # use lettre::message::Mailbox;
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let mailbox: Mailbox = "John Smith <example@email.com>".parse()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Mailbox {
    /// The name associated with the address.
    pub name: Option<String>,

    /// The email address itself.
    pub email: Address,
}

impl Mailbox {
    /// Creates a new `Mailbox` using an email address and the name of the recipient if there is one.
    ///
    /// # Examples
    ///
    /// ```
    /// use lettre::{message::Mailbox, Address};
    ///
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let address = Address::new("example", "email.com")?;
    /// let mailbox = Mailbox::new(None, address);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(name: Option<String>, email: Address) -> Self {
        Mailbox { name, email }
    }

    pub(crate) fn encode(&self, w: &mut EmailWriter<'_>) -> FmtResult {
        if let Some(name) = &self.name {
            email_encoding::headers::quoted_string::encode(name, w)?;
            w.optional_breakpoint();
            w.write_char('<')?;
        }

        w.write_str(self.email.as_ref())?;

        if self.name.is_some() {
            w.write_char('>')?;
        }

        Ok(())
    }
}

impl Display for Mailbox {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(ref name) = self.name {
            let name = name.trim();
            if !name.is_empty() {
                write_word(f, name)?;
                f.write_str(" <")?;
                self.email.fmt(f)?;
                return f.write_char('>');
            }
        }
        self.email.fmt(f)
    }
}

impl<S: Into<String>, T: Into<String>> TryFrom<(S, T)> for Mailbox {
    type Error = AddressError;

    fn try_from(header: (S, T)) -> Result<Self, Self::Error> {
        let (name, address) = header;
        Ok(Mailbox::new(Some(name.into()), address.into().parse()?))
    }
}

/*
impl<S: AsRef<&str>, T: AsRef<&str>> TryFrom<(S, T)> for Mailbox {
    type Error = AddressError;

    fn try_from(header: (S, T)) -> Result<Self, Self::Error> {
        let (name, address) = header;
        Ok(Mailbox::new(Some(name.as_ref()), address.as_ref().parse()?))
    }
}*/

impl FromStr for Mailbox {
    type Err = AddressError;

    fn from_str(src: &str) -> Result<Mailbox, Self::Err> {
        if !src.contains('<') {
            // Only an addr-spec.
            let addr = src.parse()?;
            return Ok(Mailbox::new(None, addr));
        }

        // name-addr
        // https://datatracker.ietf.org/doc/html/rfc2822#section-3.4
        let (name, angle_addr) = read_phrase(src).unwrap_or(("".into(), src));

        // https://datatracker.ietf.org/doc/html/rfc2822#section-3.4.1
        let addr_spec = angle_addr
            .trim_matches(&[' ', '\t'][..])
            .strip_prefix('<')
            .ok_or(AddressError::MissingParts)?
            .strip_suffix('>')
            .ok_or(AddressError::Unbalanced)?;

        let addr = addr_spec.parse()?;

        let name = if name.is_empty() {
            None
        } else {
            Some(name.into())
        };

        Ok(Mailbox::new(name, addr))
    }
}

/// Represents a sequence of [`Mailbox`] instances.
///
/// This type contains a sequence of mailboxes (_Some Name \<user@domain.tld\>, Another Name \<other@domain.tld\>, withoutname@domain.tld, ..._).
///
/// **NOTE**: Enable feature "serde" to be able serialize/deserialize it using [serde](https://serde.rs/).
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Mailboxes(Vec<Mailbox>);

impl Mailboxes {
    /// Creates a new list of [`Mailbox`] instances.
    ///
    /// # Examples
    ///
    /// ```
    /// use lettre::message::Mailboxes;
    /// let mailboxes = Mailboxes::new();
    /// ```
    pub fn new() -> Self {
        Mailboxes(Vec::new())
    }

    /// Adds a new [`Mailbox`] to the list, in a builder style pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use lettre::{
    ///     message::{Mailbox, Mailboxes},
    ///     Address,
    /// };
    ///
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let address = Address::new("example", "email.com")?;
    /// let mut mailboxes = Mailboxes::new().with(Mailbox::new(None, address));
    /// # Ok(())
    /// # }
    /// ```
    pub fn with(mut self, mbox: Mailbox) -> Self {
        self.0.push(mbox);
        self
    }

    /// Adds a new [`Mailbox`] to the list, in a Vec::push style pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use lettre::{
    ///     message::{Mailbox, Mailboxes},
    ///     Address,
    /// };
    ///
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let address = Address::new("example", "email.com")?;
    /// let mut mailboxes = Mailboxes::new();
    /// mailboxes.push(Mailbox::new(None, address));
    /// # Ok(())
    /// # }
    /// ```
    pub fn push(&mut self, mbox: Mailbox) {
        self.0.push(mbox);
    }

    /// Extracts the first [`Mailbox`] if it exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use lettre::{
    ///     message::{Mailbox, Mailboxes},
    ///     Address,
    /// };
    ///
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let empty = Mailboxes::new();
    /// assert!(empty.into_single().is_none());
    ///
    /// let mut mailboxes = Mailboxes::new();
    /// let address = Address::new("example", "email.com")?;
    ///
    /// mailboxes.push(Mailbox::new(None, address));
    /// assert!(mailboxes.into_single().is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_single(self) -> Option<Mailbox> {
        self.into()
    }

    /// Creates an iterator over the [`Mailbox`] instances that are currently stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use lettre::{
    ///     message::{Mailbox, Mailboxes},
    ///     Address,
    /// };
    ///
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let mut mailboxes = Mailboxes::new();
    ///
    /// let address = Address::new("example", "email.com")?;
    /// mailboxes.push(Mailbox::new(None, address));
    ///
    /// let address = Address::new("example", "email.com")?;
    /// mailboxes.push(Mailbox::new(None, address));
    ///
    /// let mut iter = mailboxes.iter();
    ///
    /// assert!(iter.next().is_some());
    /// assert!(iter.next().is_some());
    ///
    /// assert!(iter.next().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(&self) -> Iter<'_, Mailbox> {
        self.0.iter()
    }

    pub(crate) fn encode(&self, w: &mut EmailWriter<'_>) -> FmtResult {
        let mut first = true;
        for mailbox in self.iter() {
            if !mem::take(&mut first) {
                w.write_char(',')?;
                w.optional_breakpoint();
            }

            mailbox.encode(w)?;
        }

        Ok(())
    }
}

impl Default for Mailboxes {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Mailbox> for Mailboxes {
    fn from(mailbox: Mailbox) -> Self {
        Mailboxes(vec![mailbox])
    }
}

impl From<Mailboxes> for Option<Mailbox> {
    fn from(mailboxes: Mailboxes) -> Option<Mailbox> {
        mailboxes.into_iter().next()
    }
}

impl From<Vec<Mailbox>> for Mailboxes {
    fn from(vec: Vec<Mailbox>) -> Self {
        Mailboxes(vec)
    }
}

impl From<Mailboxes> for Vec<Mailbox> {
    fn from(mailboxes: Mailboxes) -> Vec<Mailbox> {
        mailboxes.0
    }
}

impl FromIterator<Mailbox> for Mailboxes {
    fn from_iter<T: IntoIterator<Item = Mailbox>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl Extend<Mailbox> for Mailboxes {
    fn extend<T: IntoIterator<Item = Mailbox>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl IntoIterator for Mailboxes {
    type Item = Mailbox;
    type IntoIter = ::std::vec::IntoIter<Mailbox>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Display for Mailboxes {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut iter = self.iter();

        if let Some(mbox) = iter.next() {
            mbox.fmt(f)?;

            for mbox in iter {
                f.write_str(", ")?;
                mbox.fmt(f)?;
            }
        }

        Ok(())
    }
}

impl FromStr for Mailboxes {
    type Err = AddressError;

    fn from_str(mut src: &str) -> Result<Self, Self::Err> {
        let mut mailboxes = Vec::new();

        if !src.is_empty() {
            // n-1 elements
            let mut skip = 0;
            while let Some(i) = src[skip..].find(',') {
                let left = &src[..skip + i];

                match left.trim().parse() {
                    Ok(mailbox) => {
                        mailboxes.push(mailbox);

                        src = &src[left.len() + ",".len()..];
                        skip = 0;
                    }
                    Err(AddressError::MissingParts) => {
                        skip = left.len() + ",".len();
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            // last element
            let mailbox = src.trim().parse()?;
            mailboxes.push(mailbox);
        }

        Ok(Mailboxes(mailboxes))
    }
}

// Note that phrase is a subset of obs-phrase so we effectively only read the latter.
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.6
// https://datatracker.ietf.org/doc/html/rfc2822#section-4.1
fn read_phrase(s: &str) -> Option<(Cow<'_, str>, &str)> {
    let (mut phrase, mut remainder) = read_word(s)?;
    while let Some((new_phrase, new_remainder)) = read_obs_word(remainder) {
        let phrase = phrase.to_mut();
        if is_fws_char(remainder.chars().next().unwrap()) {
            phrase.push(' ')
        }
        phrase.push_str(&new_phrase);
        remainder = new_remainder;
    }
    Some((phrase, remainder))
}

// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.6
fn write_word(f: &mut Formatter<'_>, s: &str) -> FmtResult {
    if s.as_bytes().iter().copied().all(is_atom_char) {
        f.write_str(s)
    } else {
        // Quoted string: https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.5
        f.write_char('"')?;
        for c in s.chars() {
            write_quoted_string_char(f, c)?;
        }
        f.write_char('"')?;

        Ok(())
    }
}

// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.6
fn read_word(s: &str) -> Option<(Cow<'_, str>, &str)> {
    read_obs_word(s).filter(|(word, _)| word != ".")
}

// obs-word is our own invention.
// It is simply `word / "."` which is useful for reading obs-phrase.
fn read_obs_word(s: &str) -> Option<(Cow<'_, str>, &str)> {
    match s.chars().next()? {
        ' ' | '\t' => read_obs_word(&s[1..]),
        '"' => read_quoted_string(&s[1..]),
        '.' => Some((".".into(), &s[1..])),
        _ => match read_atom(s) {
            ("", _) => None,
            (atom, rest) => Some((atom.into(), rest)),
        },
    }
}

// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.4
fn read_atom(s: &str) -> (&str, &str) {
    let end = s.bytes().take_while(|c| is_atom_char(*c)).count();
    s.split_at(end)
}

// https://datatracker.ietf.org/doc/html/rfc2234#section-6.1
fn is_fws_char(c: char) -> bool {
    c == ' ' || c == '\t'
}

// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.4
fn is_atom_char(c: u8) -> bool {
    matches!(c,
        b'!' |
        b'#' |
        b'$' |
        b'%' |
        b'&' |
        b'\'' |
        b'*' |
        b'+' |
        b'-' |
        b'/' |
        b'0'..=b'8' |
        b'=' |
        b'?' |
        b'A'..=b'Z' |
        b'^' |
        b'_' |
        b'`' |
        b'a'..=b'z' |
        b'{' |
        b'|' |
        b'}' |
        b'~' |

        // Not technically allowed but will be escaped into allowed characters.
        128..)
}

// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.5
fn is_quoted_string_safe(c: char) -> bool {
    is_qtext(c) || is_fws_char(c)
}

// Note: You probably want is_quoted_string_safe as it includes FWS which you rarely need to distinguish from qtext. Both can be included verbatim in a quoted string and lettre has already done unfolding.
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.5
fn is_qtext(c: char) -> bool {
    matches!(u32::from(c),
        // NO-WS-CTL: https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.1
        1..=8 | 11 | 12 | 14..=31 | 127 |

        // The rest of the US-ASCII except \ and "
        33 |
        35..=91 |
        93..=126 |

        // Not technically allowed but will be escaped into allowed characters.
        128..)
}

// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.5
fn write_quoted_string_char(f: &mut Formatter<'_>, c: char) -> FmtResult {
    match c {
        // Can not be encoded.
        '\n' | '\r' => Err(std::fmt::Error),

        c if is_quoted_string_safe(c) => f.write_char(c),

        _ => {
            // quoted-pair https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.2
            f.write_char('\\')?;
            f.write_char(c)
        }
    }
}

// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.4
fn read_quoted_string(full: &str) -> Option<(Cow<'_, str>, &str)> {
    let end = full
        .chars()
        .take_while(|c| is_quoted_string_safe(*c))
        .count();
    let (prefix, remainder) = full.split_at(end);
    if let Some(suffix) = remainder.strip_prefix('"') {
        return Some((prefix.into(), suffix));
    }

    let mut buf = prefix.to_string();
    let mut chars = remainder.chars().enumerate();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Some((buf.into(), &remainder[i + 1..])),
            '\\' => buf.push(chars.next()?.1),
            _ if is_quoted_string_safe(c) => buf.push(c),
            _ => return None,
        }
    }

    None
}

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use pretty_assertions::assert_eq;

    use super::Mailbox;

    #[test]
    fn mailbox_format_address_only() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(None, "kayo@example.com".parse().unwrap())
            ),
            "kayo@example.com"
        );
    }

    #[test]
    fn mailbox_format_address_with_name() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(Some("K.".into()), "kayo@example.com".parse().unwrap())
            ),
            "\"K.\" <kayo@example.com>"
        );
    }

    #[test]
    fn mailbox_format_address_with_comma() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(
                    Some("Last, First".into()),
                    "kayo@example.com".parse().unwrap()
                )
            ),
            r#""Last, First" <kayo@example.com>"#
        );
    }

    #[test]
    fn mailbox_format_address_with_comma_and_non_ascii() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(
                    Some("Laşt, First".into()),
                    "kayo@example.com".parse().unwrap()
                )
            ),
            r#""Laşt, First" <kayo@example.com>"#
        );
    }

    #[test]
    fn mailbox_format_address_with_comma_and_quoted_non_ascii() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(
                    Some(r#"Laşt, "First""#.into()),
                    "kayo@example.com".parse().unwrap()
                )
            ),
            r#""Laşt, \"First\"" <kayo@example.com>"#
        );
    }

    #[test]
    fn mailbox_format_address_with_angle_bracket() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(Some("<3".into()), "i@love.example".parse().unwrap())
            ),
            r#""<3" <i@love.example>"#
        );
    }

    #[test]
    fn mailbox_format_address_with_color() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(
                    Some("Chris's Wiki :: blog".into()),
                    "kayo@example.com".parse().unwrap()
                )
            ),
            r#""Chris's Wiki :: blog" <kayo@example.com>"#
        );
    }

    #[test]
    fn format_address_with_empty_name() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(Some("".into()), "kayo@example.com".parse().unwrap())
            ),
            "kayo@example.com"
        );
    }

    #[test]
    fn format_address_with_name_trim() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(Some(" K. ".into()), "kayo@example.com".parse().unwrap())
            ),
            "\"K.\" <kayo@example.com>"
        );
    }

    #[test]
    fn parse_address_only() {
        assert_eq!(
            "kayo@example.com".parse(),
            Ok(Mailbox::new(None, "kayo@example.com".parse().unwrap()))
        );
    }

    #[test]
    fn parse_address_with_name() {
        assert_eq!(
            "K. <kayo@example.com>".parse(),
            Ok(Mailbox::new(
                Some("K.".into()),
                "kayo@example.com".parse().unwrap()
            ))
        );
    }

    #[test]
    fn parse_address_with_comma() {
        assert_eq!(
            r#""<3" <i@love.example>"#.parse(),
            Ok(Mailbox::new(
                Some("<3".into()),
                "i@love.example".parse().unwrap()
            ))
        );
    }

    #[test]
    fn parse_address_with_empty_name() {
        assert_eq!(
            "<kayo@example.com>".parse(),
            Ok(Mailbox::new(None, "kayo@example.com".parse().unwrap()))
        );
    }

    #[test]
    fn parse_address_with_empty_name_trim() {
        assert_eq!(
            " <kayo@example.com>".parse(),
            Ok(Mailbox::new(None, "kayo@example.com".parse().unwrap()))
        );
    }

    #[test]
    fn parse_address_from_tuple() {
        assert_eq!(
            ("K.".to_string(), "kayo@example.com".to_string()).try_into(),
            Ok(Mailbox::new(
                Some("K.".into()),
                "kayo@example.com".parse().unwrap()
            ))
        );
    }
}
