//! Email addresses

#[cfg(feature = "serde")]
mod serde;

mod envelope;
mod types;

pub use self::envelope::Envelope;
pub use self::types::{Address, AddressError};
