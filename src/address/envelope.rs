use super::Address;
#[cfg(feature = "builder")]
use crate::message::header::{self, Headers};
#[cfg(feature = "builder")]
use crate::message::{Mailbox, Mailboxes};
use crate::Error;

/// Simple email envelope representation
///
/// We only accept mailboxes, and do not support source routes (as per RFC).
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Envelope {
    /// The envelope recipient's addresses
    ///
    /// This can not be empty.
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "serde_forward_path::deserialize")
    )]
    forward_path: Vec<Address>,
    /// The envelope sender address
    reverse_path: Option<Address>,
}

/// just like the default implementation to deserialize `Vec<Address>` but it
/// forbids **de**serializing empty lists
#[cfg(feature = "serde")]
mod serde_forward_path {
    use super::Address;
    /// dummy type required for serde
    /// see example: https://serde.rs/deserialize-map.html
    struct CustomVisitor;
    impl<'de> serde::de::Visitor<'de> for CustomVisitor {
        type Value = Vec<Address>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a non-empty list of recipient addresses")
        }

        fn visit_seq<S>(self, mut access: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            let mut seq: Vec<Address> = Vec::with_capacity(access.size_hint().unwrap_or(0));
            while let Some(key) = access.next_element()? {
                seq.push(key);
            }
            if seq.is_empty() {
                Err(serde::de::Error::invalid_length(seq.len(), &self))
            } else {
                Ok(seq)
            }
        }
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Address>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(CustomVisitor {})
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn deserializing_empty_recipient_list_returns_error() {
            assert!(
                serde_json::from_str::<crate::address::Envelope>(r#"{"forward_path": []}"#)
                    .is_err()
            );
        }
        #[test]
        fn deserializing_non_empty_recipient_list_is_ok() {
            serde_json::from_str::<crate::address::Envelope>(
                r#"{ "forward_path": [ {"user":"foo", "domain":"example.com"} ] }"#,
            )
            .unwrap();
        }
    }
}

impl Envelope {
    /// Creates a new envelope, which may fail if `to` is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lettre::address::{Address, Envelope};
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let sender = "sender@email.com".parse::<Address>()?;
    /// let recipients = vec!["to@email.com".parse::<Address>()?];
    ///
    /// let envelope = Envelope::new(Some(sender), recipients);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If `to` has no elements in it.
    pub fn new(from: Option<Address>, to: Vec<Address>) -> Result<Envelope, Error> {
        if to.is_empty() {
            return Err(Error::MissingTo);
        }
        Ok(Envelope {
            forward_path: to,
            reverse_path: from,
        })
    }

    /// Gets the destination addresses of the envelope.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lettre::address::{Address, Envelope};
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let sender = "from@email.com".parse::<Address>()?;
    /// let recipients = vec!["to@email.com".parse::<Address>()?];
    ///
    /// let envelope = Envelope::new(Some(sender), recipients.clone())?;
    /// assert_eq!(envelope.to(), recipients.as_slice());
    /// # Ok(())
    /// # }
    /// ```
    pub fn to(&self) -> &[Address] {
        self.forward_path.as_slice()
    }

    /// Gets the sender of the envelope.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lettre::address::{Address, Envelope};
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let sender = "from@email.com".parse::<Address>()?;
    /// let recipients = vec!["to@email.com".parse::<Address>()?];
    ///
    /// let envelope = Envelope::new(Some(sender), recipients.clone())?;
    /// assert!(envelope.from().is_some());
    ///
    /// let senderless = Envelope::new(None, recipients.clone())?;
    /// assert!(senderless.from().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn from(&self) -> Option<&Address> {
        self.reverse_path.as_ref()
    }

    #[cfg(feature = "smtp-transport")]
    /// Check if any of the addresses in the envelope contains non-ascii chars
    pub(crate) fn has_non_ascii_addresses(&self) -> bool {
        self.reverse_path
            .iter()
            .chain(self.forward_path.iter())
            .any(|a| !a.is_ascii())
    }
}

#[cfg(feature = "builder")]
impl TryFrom<&Headers> for Envelope {
    type Error = Error;

    fn try_from(headers: &Headers) -> Result<Self, Self::Error> {
        let from = match headers.get::<header::Sender>() {
            // If there is a Sender, use it
            Some(sender) => Some(Mailbox::from(sender).email),
            // ... else try From
            None => match headers.get::<header::From>() {
                Some(header::From(a)) => {
                    let mut from: Vec<Mailbox> = a.into();
                    if from.len() > 1 {
                        return Err(Error::TooManyFrom);
                    }
                    let from = from.pop().expect("From header has 1 Mailbox");
                    Some(from.email)
                }
                None => None,
            },
        };

        fn add_addresses_from_mailboxes(
            addresses: &mut Vec<Address>,
            mailboxes: Option<Mailboxes>,
        ) {
            if let Some(mailboxes) = mailboxes {
                addresses.extend(mailboxes.into_iter().map(|mb| mb.email));
            }
        }
        let mut to = vec![];
        add_addresses_from_mailboxes(&mut to, headers.get::<header::To>().map(|h| h.0));
        add_addresses_from_mailboxes(&mut to, headers.get::<header::Cc>().map(|h| h.0));
        add_addresses_from_mailboxes(&mut to, headers.get::<header::Bcc>().map(|h| h.0));

        Self::new(from, to)
    }
}
