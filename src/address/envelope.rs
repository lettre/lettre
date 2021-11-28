use super::Address;
#[cfg(feature = "builder")]
use crate::message::Mailboxes;
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
    forward_path: Vec<Address>,
    /// The envelope sender address
    reverse_path: Option<Address>,
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

    #[cfg(feature = "builder")]
    pub(crate) fn build(from: Address, to: Mailboxes, cc: Mailboxes, bcc: Mailboxes) -> Self {
        let mut forward_path = Vec::new();

        forward_path.extend(to.into_iter().map(|mailbox| mailbox.email));
        forward_path.extend(cc.into_iter().map(|mailbox| mailbox.email));
        forward_path.extend(bcc.into_iter().map(|mailbox| mailbox.email));

        Self {
            forward_path,
            reverse_path: Some(from),
        }
    }
}
