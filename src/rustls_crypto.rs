use std::sync::Arc;

use rustls::crypto::CryptoProvider;

pub(crate) fn crypto_provider() -> Arc<CryptoProvider> {
    #[cfg(any(feature = "aws-lc-rs", feature = "ring"))]
    {
        CryptoProvider::get_default().cloned().unwrap_or_else(|| {
            #[cfg(feature = "aws-lc-rs")]
            let provider = rustls::crypto::aws_lc_rs::default_provider();
            #[cfg(all(not(feature = "aws-lc-rs"), feature = "ring"))]
            let provider = rustls::crypto::ring::default_provider();

            Arc::new(provider)
        })
    }

    #[cfg(not(any(feature = "aws-lc-rs", feature = "ring")))]
    {
        CryptoProvider::get_default()
            .cloned()
            .expect("No rustls crypto provider configured. When using the `rustls-no-provider` feature, a crypto provider must be installed before using lettre. For example:\n\n    use rustls::crypto::CryptoProvider;\n    CryptoProvider::install_default(your_provider).expect(\"Failed to install crypto provider\");\n")
    }
}
