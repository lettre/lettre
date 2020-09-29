//! Representation of an email address

use idna::domain_to_ascii;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    ffi::OsStr,
    fmt::{Display, Formatter, Result as FmtResult},
    net::IpAddr,
    str::FromStr,
};

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
/// # use lettre::Address;
/// let address = Address::new("example", "email.com").unwrap();
/// ```
///
/// You can also create an `Address` from a string literal by parsing it:
///
/// ```
/// use std::str::FromStr;
/// # use lettre::Address;
/// let address = Address::from_str("example@email.com").unwrap();
/// ```
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Address {
    /// Complete address
    serialized: String,
    /// Index into `serialized` before the '@'
    at_start: usize,
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
    /// let address = Address::new("example", "email.com").unwrap();
    /// let expected: Address = "example@email.com".parse().unwrap();
    /// assert_eq!(expected, address);
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
    /// let address = Address::new("example", "email.com").unwrap();
    /// assert_eq!("example", address.user());
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
    /// let address = Address::new("example", "email.com").unwrap();
    /// assert_eq!("email.com", address.domain());
    /// ```
    pub fn domain(&self) -> &str {
        &self.serialized[self.at_start + 1..]
    }

    fn check_user(user: &str) -> Result<(), AddressError> {
        if USER_RE.is_match(user) {
            Ok(())
        } else {
            Err(AddressError::InvalidUser)
        }
    }

    fn check_domain(domain: &str) -> Result<(), AddressError> {
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
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(&self.serialized)
    }
}

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(val: &str) -> Result<Self, AddressError> {
        let mut parts = val.rsplitn(2, '@');
        let domain = parts.next().ok_or(AddressError::MissingParts)?;
        let user = parts.next().ok_or(AddressError::MissingParts)?;

        Address::check_user(user)?;
        Address::check_domain(domain)?;
        Ok(Address {
            serialized: val.into(),
            at_start: user.len(),
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AddressError {
    MissingParts,
    Unbalanced,
    InvalidUser,
    InvalidDomain,
    InvalidUtf8b,
}

impl Error for AddressError {}

impl Display for AddressError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            AddressError::MissingParts => f.write_str("Missing domain or user"),
            AddressError::Unbalanced => f.write_str("Unbalanced angle bracket"),
            AddressError::InvalidUser => f.write_str("Invalid email user"),
            AddressError::InvalidDomain => f.write_str("Invalid email domain"),
            AddressError::InvalidUtf8b => f.write_str("Invalid UTF8b data"),
        }
    }
}

#[cfg(feature = "serde")]
mod serde {
    use crate::address::Address;
    use serde::{
        de::{Deserializer, Error as DeError, MapAccess, Visitor},
        ser::Serializer,
        Deserialize, Serialize,
    };
    use std::fmt::{Formatter, Result as FmtResult};

    impl Serialize for Address {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.as_ref())
        }
    }

    impl<'de> Deserialize<'de> for Address {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            enum Field {
                User,
                Domain,
            };

            const FIELDS: &[&str] = &["user", "domain"];

            impl<'de> Deserialize<'de> for Field {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct FieldVisitor;

                    impl<'de> Visitor<'de> for FieldVisitor {
                        type Value = Field;

                        fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                            formatter.write_str("'user' or 'domain'")
                        }

                        fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where
                            E: DeError,
                        {
                            match value {
                                "user" => Ok(Field::User),
                                "domain" => Ok(Field::Domain),
                                _ => Err(DeError::unknown_field(value, FIELDS)),
                            }
                        }
                    }

                    deserializer.deserialize_identifier(FieldVisitor)
                }
            }

            struct AddressVisitor;

            impl<'de> Visitor<'de> for AddressVisitor {
                type Value = Address;

                fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                    formatter.write_str("email address string or object")
                }

                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: DeError,
                {
                    s.parse().map_err(DeError::custom)
                }

                fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                where
                    V: MapAccess<'de>,
                {
                    let mut user = None;
                    let mut domain = None;
                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::User => {
                                if user.is_some() {
                                    return Err(DeError::duplicate_field("user"));
                                }
                                let val = map.next_value()?;
                                Address::check_user(val).map_err(DeError::custom)?;
                                user = Some(val);
                            }
                            Field::Domain => {
                                if domain.is_some() {
                                    return Err(DeError::duplicate_field("domain"));
                                }
                                let val = map.next_value()?;
                                Address::check_domain(val).map_err(DeError::custom)?;
                                domain = Some(val);
                            }
                        }
                    }
                    let user: &str = user.ok_or_else(|| DeError::missing_field("user"))?;
                    let domain: &str = domain.ok_or_else(|| DeError::missing_field("domain"))?;
                    Ok(Address::new(user, domain).unwrap())
                }
            }

            deserializer.deserialize_any(AddressVisitor)
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
