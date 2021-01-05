use std::fmt::{Formatter, Result as FmtResult};

use serde::{
    de::{Deserializer, Error as DeError, MapAccess, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};

use super::Address;

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
        }

        const FIELDS: &[&str] = &["user", "domain"];

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
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

            fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
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
