#[cfg(feature = "rustls-tls")]
use std::sync::Arc;

#[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
use crate::transport::smtp::error::Error;

#[cfg(feature = "native-tls")]
use native_tls::{Protocol, TlsConnector};
#[cfg(feature = "rustls-tls")]
use rustls::{
    Certificate, ClientConfig, RootCertStore, ServerCertVerified, ServerCertVerifier, TLSError,
};
#[cfg(feature = "rustls-tls")]
use webpki::DNSNameRef;

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
    pub(super) domain: String,
}

/// Builder for `TlsParameters`
#[derive(Debug, Clone)]
pub struct TlsParametersBuilder {
    domain: String,
    accept_invalid_hostnames: bool,
    accept_invalid_certs: bool,
}

impl TlsParametersBuilder {
    /// Creates a new builder for `TlsParameters`
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            accept_invalid_hostnames: false,
            accept_invalid_certs: false,
        }
    }

    /// Controls whether certificates with an invalid hostname are accepted
    ///
    /// Defaults to `false`.
    ///
    /// # Warning
    ///
    /// You should think very carefully before using this method.
    /// If hostname verification is disabled *any* valid certificate,
    /// including those from other sites, are trusted.
    ///
    /// This method introduces significant vulnerabilities to man-in-the-middle attacks.
    ///
    /// Hostname verification can only be disabled with the `native-tls` TLS backend.
    #[cfg(feature = "native-tls")]
    pub fn dangerous_accept_invalid_hostnames(
        &mut self,
        accept_invalid_hostnames: bool,
    ) -> &mut Self {
        self.accept_invalid_hostnames = accept_invalid_hostnames;
        self
    }

    /// Controls whether invalid certificates are accepted
    ///
    /// Defaults to `false`.
    ///
    /// # Warning
    ///
    /// You should think very carefully before using this method.
    /// If certificate verification is disabled, *any* certificate
    /// is trusted for use, including:
    ///
    /// * Self signed certificates
    /// * Certificates from different hostnames
    /// * Expired certificates
    ///
    /// This method should only be used as a last resort, as it introduces
    /// significant vulnerabilities to man-in-the-middle attacks.
    pub fn dangerous_accept_invalid_certs(&mut self, accept_invalid_certs: bool) -> &mut Self {
        self.accept_invalid_certs = accept_invalid_certs;
        self
    }

    /// Creates a new `TlsParameters` using native-tls or rustls
    /// depending on which one is available
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    pub fn build(self) -> Result<TlsParameters, Error> {
        #[cfg(feature = "native-tls")]
        return self.build_native();

        #[cfg(not(feature = "native-tls"))]
        return self.build_rustls();
    }

    #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
    pub(crate) fn build_tokio02(self) -> Result<TlsParameters, Error> {
        #[cfg(feature = "tokio02-native-tls")]
        return self.build_native();

        #[cfg(not(feature = "tokio02-native-tls"))]
        return self.build_rustls();
    }

    /// Creates a new `TlsParameters` using native-tls with the provided configuration
    #[cfg(feature = "native-tls")]
    pub fn build_native(self) -> Result<TlsParameters, Error> {
        let mut tls_builder = TlsConnector::builder();
        tls_builder.danger_accept_invalid_hostnames(self.accept_invalid_hostnames);
        tls_builder.danger_accept_invalid_certs(self.accept_invalid_certs);
        tls_builder.min_protocol_version(Some(DEFAULT_TLS_MIN_PROTOCOL));
        let connector = tls_builder.build()?;
        Ok(TlsParameters {
            connector: InnerTlsParameters::NativeTls(connector),
            domain: self.domain,
        })
    }

    /// Creates a new `TlsParameters` using rustls with the provided configuration
    #[cfg(feature = "rustls-tls")]
    pub fn build_rustls(self) -> Result<TlsParameters, Error> {
        use webpki_roots::TLS_SERVER_ROOTS;

        let mut tls = ClientConfig::new();
        if self.accept_invalid_certs {
            tls.dangerous()
                .set_certificate_verifier(Arc::new(InvalidCertsVerifier {}));
        }
        tls.root_store.add_server_trust_anchors(&TLS_SERVER_ROOTS);
        Ok(TlsParameters {
            connector: InnerTlsParameters::RustlsTls(tls),
            domain: self.domain,
        })
    }
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
        TlsParametersBuilder::new(domain).build()
    }

    pub fn builder(domain: String) -> TlsParametersBuilder {
        TlsParametersBuilder::new(domain)
    }

    /// Creates a new `TlsParameters` using native-tls
    #[cfg(feature = "native-tls")]
    pub fn new_native(domain: String) -> Result<Self, Error> {
        TlsParametersBuilder::new(domain).build_native()
    }

    /// Creates a new `TlsParameters` using rustls
    #[cfg(feature = "rustls-tls")]
    pub fn new_rustls(domain: String) -> Result<Self, Error> {
        TlsParametersBuilder::new(domain).build_rustls()
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }
}

#[cfg(feature = "rustls-tls")]
struct InvalidCertsVerifier;

#[cfg(feature = "rustls-tls")]
impl ServerCertVerifier for InvalidCertsVerifier {
    fn verify_server_cert(
        &self,
        _roots: &RootCertStore,
        _presented_certs: &[Certificate],
        _dns_name: DNSNameRef<'_>,
        _ocsp_response: &[u8],
    ) -> Result<ServerCertVerified, TLSError> {
        Ok(ServerCertVerified::assertion())
    }
}
