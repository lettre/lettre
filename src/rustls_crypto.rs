#[cfg(feature = "aws-lc-rs")]
pub(crate) use rustls::crypto::aws_lc_rs::default_provider as crypto_provider;
#[cfg(not(feature = "aws-lc-rs"))]
pub(crate) use rustls::crypto::ring::default_provider as crypto_provider;
