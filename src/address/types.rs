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
/// This type contains email in canonical form (_user@domain.tld_).
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
    /// Creates a new email address from a string without checking it.
    ///
    /// # Panics
    /// Will panic if @ is not present in the string
    pub fn new_unchecked(serialized: String) -> Self {
        let at_start = serialized.chars().position(|c| c =='@').unwrap();

        Self {
            serialized,
            at_start,
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
        if EmailAddress::is_valid_local_part(user) {
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
    fn check_parts() {
        assert!(Address::check_user("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").is_err());
        assert!(
            Address::check_domain("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.com").is_err()
        );
    }

    #[test]
    fn test_new_unchecked() {
        let addr = Address::new_unchecked("something@example.com".to_owned());
        assert_eq!(addr.user(), "something");
        assert_eq!(addr.domain(), "example.com");

        assert_eq!(addr, Address::new("something", "example.com").unwrap());
    }

    #[test]
    #[should_panic]
    fn test_new_unchecked_panic() {
        Address::new_unchecked("somethingexample.com".to_owned());
    }
}
