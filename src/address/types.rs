//! Representation of an email address

use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    ffi::OsStr,
    fmt::{Display, Formatter, Result as FmtResult},
    net::IpAddr,
    str::FromStr,
};

use idna::domain_to_ascii;
use once_cell::sync::Lazy;
use regex::Regex;

/// Represents an email address with a user and a domain name.
///
/// This type contains email in canonical form (_user@domain.tld_).
///
/// **NOTE**: Enable feature "serde" to be able serialize/deserialize it using [serde](https://serde.rs/).
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

// Regex from the specs
// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
// It will mark esoteric email addresses like quoted string as invalid
static USER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?i)[a-z0-9.!#$%&'*+/=?^_`{|}~-]+\z").unwrap());
static DOMAIN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*$",
    )
    .unwrap()
});
// literal form, ipv4 or ipv6 address (SMTP 4.1.3)
static LITERAL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\[([A-f0-9:\.]+)\]\z").unwrap());

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
        if USER_RE.is_match(user) {
            Ok(())
        } else {
            Err(AddressError::InvalidUser)
        }
    }

    pub(super) fn check_domain(domain: &str) -> Result<(), AddressError> {
        Address::check_domain_ascii(domain).or_else(|_| {
            domain_to_ascii(domain)
                .map_err(|_| AddressError::InvalidDomain)
                .and_then(|domain| Address::check_domain_ascii(&domain))
        })
    }

    fn check_domain_ascii(domain: &str) -> Result<(), AddressError> {
        if DOMAIN_RE.is_match(domain) {
            return Ok(());
        }

        if let Some(caps) = LITERAL_RE.captures(domain) {
            if let Some(cap) = caps.get(1) {
                if cap.as_str().parse::<IpAddr>().is_ok() {
                    return Ok(());
                }
            }
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

        let serialized = format!("{}@{}", user, domain);
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

fn check_address(val: &str) -> Result<usize, AddressError> {
    let mut parts = val.rsplitn(2, '@');
    let domain = parts.next().ok_or(AddressError::MissingParts)?;
    let user = parts.next().ok_or(AddressError::MissingParts)?;

    Address::check_user(user)?;
    Address::check_domain(domain)?;
    Ok(user.len())
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
}

impl Error for AddressError {}

impl Display for AddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            AddressError::MissingParts => f.write_str("Missing domain or user"),
            AddressError::Unbalanced => f.write_str("Unbalanced angle bracket"),
            AddressError::InvalidUser => f.write_str("Invalid email user"),
            AddressError::InvalidDomain => f.write_str("Invalid email domain"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_address() {
        let addr_str = "something@example.com";
        let addr = Address::from_str(addr_str).unwrap();
        let addr2 = Address::new("something", "example.com").unwrap();
        assert_eq!(addr, addr2);
        assert_eq!(addr.user(), "something");
        assert_eq!(addr.domain(), "example.com");
        assert_eq!(addr2.user(), "something");
        assert_eq!(addr2.domain(), "example.com");
    }
}
