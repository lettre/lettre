//! Representation of an email address

use std::{
    error::Error,
    ffi::OsStr,
    fmt::{Display, Formatter, Result as FmtResult},
    net::IpAddr,
    str::FromStr,
};

use email_address::EmailAddress;
use idna::domain_to_ascii;

/// Represents an email address with a user and a domain name.
///
/// This type holds a valid email address (_user@domain.tld_) as it was
/// given: it is not brought to a canonical form, so that `a@EXAMPLE.com`
/// and `a@example.com` are two different addresses.
///
/// **NOTE**: Enable feature "serde" to be able to serialize/deserialize it using [serde](https://serde.rs/).
///
/// # Examples
///
/// You can create an `Address` from a user and a domain:
///
/// ```
/// use lettre::Address;
///
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let address = Address::new("user", "email.com")?;
/// assert_eq!(address.user(), "user");
/// assert_eq!(address.domain(), "email.com");
/// # Ok(())
/// # }
/// ```
///
/// You can also create an `Address` from a string literal by parsing it:
///
/// ```
/// use lettre::Address;
///
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let address = "user@email.com".parse::<Address>()?;
/// assert_eq!(address.user(), "user");
/// assert_eq!(address.domain(), "email.com");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Address {
    /// Complete address
    serialized: String,
    /// Index into `serialized` before the '@'
    at_start: usize,
}

impl Address {
    /// Creates a new email address from a user and domain.
    ///
    /// # Examples
    ///
    /// ```
    /// use lettre::Address;
    ///
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let address = Address::new("user", "email.com")?;
    /// let expected = "user@email.com".parse::<Address>()?;
    /// assert_eq!(expected, address);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<U: AsRef<str>, D: AsRef<str>>(user: U, domain: D) -> Result<Self, AddressError> {
        (user, domain).try_into()
    }

    /// Creates a new email address from a user and domain, without validation.
    ///
    /// # Danger
    ///
    /// Use this method only if you are completely sure your input is _known_ and valid, but rejected by the parser.
    /// Do not use this method if the contents of either `user` or `domain` are unknown and/or obtained from user input.
    ///
    /// Prefer using [`Address::new()`] instead if possible.
    ///
    /// The message builder reads the addresses of a message back from its
    /// headers. A header with an address that [`Address::new()`] would
    /// reject can't be read back: its recipients are left out of the
    /// envelope made for the message, which is only an error when none is
    /// left.
    ///
    /// # Examples
    ///
    /// ```
    /// use lettre::Address;
    ///
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let address = Address::new_dangerous(
    ///     "a_very_long_email_address_that_would_normally_be_rejected_but_it_is_valid_trust_me",
    ///     "email.com",
    /// );
    /// assert_eq!(
    ///     address.user(),
    ///     "a_very_long_email_address_that_would_normally_be_rejected_but_it_is_valid_trust_me"
    /// );
    /// assert_eq!(address.domain(), "email.com");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_dangerous<U: AsRef<str>, D: AsRef<str>>(user: U, domain: D) -> Self {
        // same body as TryFrom impl without checks
        let user = user.as_ref();
        let domain = domain.as_ref();
        let serialized = format!("{user}@{domain}");
        Address {
            serialized,
            at_start: user.len(),
        }
    }

    /// Gets the user portion of the `Address`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lettre::Address;
    ///
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let address = Address::new("user", "email.com")?;
    /// assert_eq!(address.user(), "user");
    /// # Ok(())
    /// # }
    /// ```
    pub fn user(&self) -> &str {
        &self.serialized[..self.at_start]
    }

    /// Gets the domain portion of the `Address`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lettre::Address;
    ///
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let address = Address::new("user", "email.com")?;
    /// assert_eq!(address.domain(), "email.com");
    /// # Ok(())
    /// # }
    /// ```
    pub fn domain(&self) -> &str {
        &self.serialized[self.at_start + 1..]
    }

    pub(super) fn check_user(user: &str) -> Result<(), AddressError> {
        if EmailAddress::is_valid_local_part(user) && !user.contains(is_non_ascii_space_or_control)
        {
            Ok(())
        } else {
            Err(AddressError::InvalidUser)
        }
    }

    pub(super) fn check_domain(domain: &str) -> Result<(), AddressError> {
        if domain.contains(is_non_ascii_space_or_control) {
            return Err(AddressError::InvalidDomain);
        }

        Address::check_domain_ascii(domain).or_else(|_| {
            domain_to_ascii(domain)
                .map_err(|_| AddressError::InvalidDomain)
                .and_then(|domain| Address::check_domain_ascii(&domain))
        })
    }

    fn check_domain_ascii(domain: &str) -> Result<(), AddressError> {
        // Domain
        if EmailAddress::is_valid_domain(domain) {
            return Ok(());
        }

        // IP
        let ip = domain
            .strip_prefix('[')
            .and_then(|ip| ip.strip_suffix(']'))
            .unwrap_or(domain);

        if ip.parse::<IpAddr>().is_ok() {
            return Ok(());
        }

        Err(AddressError::InvalidDomain)
    }

    #[cfg(feature = "smtp-transport")]
    /// Check if the address contains non-ascii chars
    pub(super) fn is_ascii(&self) -> bool {
        self.serialized.is_ascii()
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.serialized)
    }
}

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(val: &str) -> Result<Self, AddressError> {
        let at_start = check_address(val)?;
        Ok(Address {
            serialized: val.into(),
            at_start,
        })
    }
}

impl<U, D> TryFrom<(U, D)> for Address
where
    U: AsRef<str>,
    D: AsRef<str>,
{
    type Error = AddressError;

    fn try_from((user, domain): (U, D)) -> Result<Self, Self::Error> {
        let user = user.as_ref();
        Address::check_user(user)?;

        let domain = domain.as_ref();
        Address::check_domain(domain)?;

        let serialized = format!("{user}@{domain}");
        Ok(Address {
            serialized,
            at_start: user.len(),
        })
    }
}

impl TryFrom<String> for Address {
    type Error = AddressError;

    fn try_from(serialized: String) -> Result<Self, AddressError> {
        let at_start = check_address(&serialized)?;
        Ok(Address {
            serialized,
            at_start,
        })
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        &self.serialized
    }
}

impl AsRef<OsStr> for Address {
    fn as_ref(&self) -> &OsStr {
        self.serialized.as_ref()
    }
}

/// White space and control characters outside of ASCII.
///
/// The mailbox parser takes the white space around an address as not being
/// part of it, so an address starting with it would be read back as another
/// one. `email_address` only rejects these characters up to version 0.2.9.
fn is_non_ascii_space_or_control(c: char) -> bool {
    !c.is_ascii() && (c.is_whitespace() || c.is_control())
}

fn check_address(val: &str) -> Result<usize, AddressError> {
    let mut parts = val.rsplitn(2, '@');
    let domain = parts.next().ok_or(AddressError::MissingParts)?;
    let user = parts.next().ok_or(AddressError::MissingParts)?;

    Address::check_user(user)?;
    Address::check_domain(domain)?;
    Ok(user.len())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
/// Errors in email addresses parsing
pub enum AddressError {
    /// Missing domain or user
    MissingParts,
    /// Unbalanced angle bracket
    Unbalanced,
    /// Invalid email user
    InvalidUser,
    /// Invalid email domain
    InvalidDomain,
    /// Invalid input found
    InvalidInput,
}

impl Error for AddressError {}

impl Display for AddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            AddressError::MissingParts => f.write_str("Missing domain or user"),
            AddressError::Unbalanced => f.write_str("Unbalanced angle bracket"),
            AddressError::InvalidUser => f.write_str("Invalid email user"),
            AddressError::InvalidDomain => f.write_str("Invalid email domain"),
            AddressError::InvalidInput => f.write_str("Invalid input"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_address() {
        let addr_str = "something@example.com";
        let addr = Address::from_str(addr_str).unwrap();
        let addr2 = Address::new("something", "example.com").unwrap();
        assert_eq!(addr, addr2);
        assert_eq!(addr.user(), "something");
        assert_eq!(addr.domain(), "example.com");
        assert_eq!(addr2.user(), "something");
        assert_eq!(addr2.domain(), "example.com");
    }

    #[test]
    fn ascii_address_ipv4() {
        let addr_str = "something@1.1.1.1";
        let addr = Address::from_str(addr_str).unwrap();
        let addr2 = Address::new("something", "1.1.1.1").unwrap();
        assert_eq!(addr, addr2);
        assert_eq!(addr.user(), "something");
        assert_eq!(addr.domain(), "1.1.1.1");
        assert_eq!(addr2.user(), "something");
        assert_eq!(addr2.domain(), "1.1.1.1");
    }

    #[test]
    fn ascii_address_ipv6() {
        let addr_str = "something@[2606:4700:4700::1111]";
        let addr = Address::from_str(addr_str).unwrap();
        let addr2 = Address::new("something", "[2606:4700:4700::1111]").unwrap();
        assert_eq!(addr, addr2);
        assert_eq!(addr.user(), "something");
        assert_eq!(addr.domain(), "[2606:4700:4700::1111]");
        assert_eq!(addr2.user(), "something");
        assert_eq!(addr2.domain(), "[2606:4700:4700::1111]");
    }

    #[test]
    fn non_ascii_white_space() {
        // NO-BREAK SPACE, NEXT LINE, LINE SEPARATOR and IDEOGRAPHIC SPACE
        for c in ['\u{a0}', '\u{85}', '\u{2028}', '\u{3000}'] {
            assert!(Address::from_str(&format!("{c}something@example.com")).is_err());
            assert!(Address::from_str(&format!("some{c}thing@example.com")).is_err());
            assert!(Address::from_str(&format!("\"some{c}thing\"@example.com")).is_err());
            assert!(Address::new("something", format!("exa{c}mple.com")).is_err());
            assert!(Address::new("something", format!("[192.0.2.1{c}]")).is_err());
            // `email_address` up to 0.2.9 rejects all of the above by itself
            assert!(is_non_ascii_space_or_control(c));
        }

        // Other characters outside of ASCII, and the white space of ASCII
        // in a quoted local part, are fine
        assert!(Address::from_str("s\u{f6}mething@example.com").is_ok());
        assert!(Address::from_str("\"some thing\"@example.com").is_ok());
        assert!(!is_non_ascii_space_or_control('\u{f6}'));
        assert!(!is_non_ascii_space_or_control(' '));
    }

    #[test]
    fn check_parts() {
        assert!(Address::check_user("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").is_err());
        assert!(
            Address::check_domain("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.com").is_err()
        );
    }

    #[test]
    fn test_unchecked() {
        let user =
            "a_very_long_email_address_that_would_normally_be_rejected_but_it_is_valid_trust_me";
        let domain = "maybe_valid.com";
        let address = Address::new_dangerous(user, domain);
        assert_eq!(address.user(), user);
        assert_eq!(address.domain(), domain);
    }
}
