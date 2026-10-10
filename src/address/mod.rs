//! Email addresses

#[cfg(feature = "serde")]
mod serde;

mod envelope;
mod types;

pub use self::{
    envelope::{DsnNotify, DsnRet, Envelope},
    types::{Address, AddressError},
};
