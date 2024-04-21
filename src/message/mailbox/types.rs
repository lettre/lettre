use std::{
    fmt::{Display, Formatter, Result as FmtResult, Write},
    mem,
    slice::Iter,
    str::FromStr,
};

use chumsky::prelude::*;
use email_encoding::headers::writer::EmailWriter;

use super::parsers;
use crate::address::{Address, AddressError};

/// Represents an email address with an optional name for the sender/recipient.
///
/// This type contains email address and the sender/recipient name (_Some Name \<user@domain.tld\>_ or _withoutname@domain.tld_).
///
/// **NOTE**: Enable feature "serde" to be able to serialize/deserialize it using [serde](https://serde.rs/).
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
            w.space();
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
        if let Some(name) = &self.name {
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

impl FromStr for Mailbox {
    type Err = AddressError;

    fn from_str(src: &str) -> Result<Mailbox, Self::Err> {
        let (name, (user, domain)) = parsers::mailbox().parse(src).map_err(|_errs| {
            // TODO: improve error management
            AddressError::InvalidInput
        })?;

        let mailbox = Mailbox::new(name, Address::new(user, domain)?);

        Ok(mailbox)
    }
}

impl From<Address> for Mailbox {
    fn from(value: Address) -> Self {
        Self::new(None, value)
    }
}

/// Represents a sequence of [`Mailbox`] instances.
///
/// This type contains a sequence of mailboxes (_Some Name \<user@domain.tld\>, Another Name \<other@domain.tld\>, withoutname@domain.tld, ..._).
///
/// **NOTE**: Enable feature "serde" to be able to serialize/deserialize it using [serde](https://serde.rs/).
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
                w.space();
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

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let mut mailboxes = Vec::new();

        let parsed_mailboxes = parsers::mailbox_list().parse(src).map_err(|_errs| {
            // TODO: improve error management
            AddressError::InvalidInput
        })?;

        for (name, (user, domain)) in parsed_mailboxes {
            mailboxes.push(Mailbox::new(name, Address::new(user, domain)?))
        }

        Ok(Mailboxes(mailboxes))
    }
}

// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.6
fn write_word(f: &mut Formatter<'_>, s: &str) -> FmtResult {
    if s.as_bytes().iter().copied().all(is_valid_atom_char) {
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

// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.4
fn is_valid_atom_char(c: u8) -> bool {
    matches!(c,
		// Not really allowed but can be inserted between atoms.
		b'\t' |
		b' ' |

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
		128..=255)
}

// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.5
fn write_quoted_string_char(f: &mut Formatter<'_>, c: char) -> FmtResult {
    match c {
        // Can not be encoded.
        '\n' | '\r' => Err(std::fmt::Error),

        // Note, not qcontent but can be put before or after any qcontent.
        '\t' | ' ' => f.write_char(c),

        c if match c as u32 {
            // NO-WS-CTL: https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.1
            1..=8 | 11 | 12 | 14..=31 | 127 |

            // The rest of the US-ASCII except \ and "
            33 |
            35..=91 |
            93..=126 |

            // Non-ascii characters will be escaped separately later.
            128.. => true,
            _ => false,
        } =>
        {
            f.write_char(c)
        }

        _ => {
            // quoted-pair https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.2
            f.write_char('\\')?;
            f.write_char(c)
        }
    }
}

#[cfg(test)]
mod test {
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
            ("K.".to_owned(), "kayo@example.com".to_owned()).try_into(),
            Ok(Mailbox::new(
                Some("K.".into()),
                "kayo@example.com".parse().unwrap()
            ))
        );
    }
}
