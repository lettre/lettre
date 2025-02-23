use std::sync::Arc;

use rustls::crypto::CryptoProvider;

pub(crate) fn crypto_provider() -> Arc<CryptoProvider> {
    CryptoProvider::get_default().cloned().unwrap_or_else(|| {
        #[cfg(feature = "aws-lc-rs")]
        let provider = rustls::crypto::aws_lc_rs::default_provider();
        #[cfg(not(feature = "aws-lc-rs"))]
        let provider = rustls::crypto::ring::default_provider();

        Arc::new(provider)
    })
}
