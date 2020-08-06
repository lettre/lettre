#[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
use crate::transport::smtp::error::Error;

#[cfg(feature = "native-tls")]
use native_tls::{Protocol, TlsConnector};
#[cfg(feature = "rustls-tls")]
use rustls::ClientConfig;

/// Accepted protocols by default.
/// This removes TLS 1.0 and 1.1 compared to tls-native defaults.
// This is also rustls' default behavior
#[cfg(feature = "native-tls")]
const DEFAULT_TLS_MIN_PROTOCOL: Protocol = Protocol::Tlsv12;

/// How to apply TLS to a client connection
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub enum Tls {
    /// Insecure connection only (for testing purposes)
    None,
    /// Start with insecure connection and use `STARTTLS` when available
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    Opportunistic(TlsParameters),
    /// Start with insecure connection and require `STARTTLS`
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    Required(TlsParameters),
    /// Use TLS wrapped connection
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    Wrapper(TlsParameters),
}

/// Parameters to use for secure clients
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct TlsParameters {
    pub(crate) connector: InnerTlsParameters,
    /// The domain name which is expected in the TLS certificate from the server
    domain: String,
}

#[derive(Clone)]
pub enum InnerTlsParameters {
    #[cfg(feature = "native-tls")]
    NativeTls(TlsConnector),
    #[cfg(feature = "rustls-tls")]
    RustlsTls(ClientConfig),
}

impl TlsParameters {
    /// Creates a new `TlsParameters` using native-tls or rustls
    /// depending on which one is available
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    pub fn new(domain: String) -> Result<Self, Error> {
        #[cfg(feature = "native-tls")]
        return Self::new_native(domain);

        #[cfg(not(feature = "native-tls"))]
        return Self::new_rustls(domain);
    }

    /// Creates a new `TlsParameters` using native-tls
    #[cfg(feature = "native-tls")]
    pub fn new_native(domain: String) -> Result<Self, Error> {
        let mut tls_builder = TlsConnector::builder();
        tls_builder.min_protocol_version(Some(DEFAULT_TLS_MIN_PROTOCOL));
        let connector = tls_builder.build()?;
        Ok(Self {
            connector: InnerTlsParameters::NativeTls(connector),
            domain,
        })
    }

    /// Creates a new `TlsParameters` using rustls
    #[cfg(feature = "rustls-tls")]
    pub fn new_rustls(domain: String) -> Result<Self, Error> {
        use webpki_roots::TLS_SERVER_ROOTS;

        let mut tls = ClientConfig::new();
        tls.root_store.add_server_trust_anchors(&TLS_SERVER_ROOTS);
        Ok(Self {
            connector: InnerTlsParameters::RustlsTls(tls),
            domain,
        })
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }
}
