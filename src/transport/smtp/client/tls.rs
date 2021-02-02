#[cfg(feature = "rustls-tls")]
use std::sync::Arc;

#[cfg(feature = "native-tls")]
use native_tls::{Protocol, TlsConnector};
#[cfg(feature = "rustls-tls")]
use rustls::{ClientConfig, RootCertStore, ServerCertVerified, ServerCertVerifier, TLSError};
#[cfg(feature = "rustls-tls")]
use webpki::DNSNameRef;

use crate::transport::smtp::error::Error;

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
#[derive(Clone)]
pub struct TlsParametersBuilder {
    domain: String,
    root_certs: Vec<Certificate>,
    accept_invalid_hostnames: bool,
    accept_invalid_certs: bool,
}

impl TlsParametersBuilder {
    /// Creates a new builder for `TlsParameters`
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            root_certs: Vec::new(),
            accept_invalid_hostnames: false,
            accept_invalid_certs: false,
        }
    }

    /// Add a custom root certificate
    ///
    /// Can be used to safely connect to a server using a self signed certificate, for example.
    pub fn add_root_certificate(mut self, cert: Certificate) -> Self {
        self.root_certs.push(cert);
        self
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
    #[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
    pub fn dangerous_accept_invalid_hostnames(mut self, accept_invalid_hostnames: bool) -> Self {
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
    pub fn dangerous_accept_invalid_certs(mut self, accept_invalid_certs: bool) -> Self {
        self.accept_invalid_certs = accept_invalid_certs;
        self
    }

    /// Creates a new `TlsParameters` using native-tls or rustls
    /// depending on which one is available
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "native-tls", feature = "rustls-tls"))))]
    // TODO: remove below line once native-tls is supported with async-std
    #[allow(unreachable_code)]
    pub fn build(self) -> Result<TlsParameters, Error> {
        // TODO: remove once native-tls is supported with async-std
        #[cfg(all(feature = "rustls-tls", feature = "async-std1"))]
        return self.build_rustls();

        #[cfg(feature = "native-tls")]
        return self.build_native();

        #[cfg(not(feature = "native-tls"))]
        return self.build_rustls();
    }

    /// Creates a new `TlsParameters` using native-tls with the provided configuration
    #[cfg(feature = "native-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
    pub fn build_native(self) -> Result<TlsParameters, Error> {
        let mut tls_builder = TlsConnector::builder();

        for cert in self.root_certs {
            tls_builder.add_root_certificate(cert.native_tls);
        }
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
    #[cfg_attr(docsrs, doc(cfg(feature = "rustls-tls")))]
    pub fn build_rustls(self) -> Result<TlsParameters, Error> {
        use webpki_roots::TLS_SERVER_ROOTS;

        let mut tls = ClientConfig::new();

        for cert in self.root_certs {
            for rustls_cert in cert.rustls {
                tls.root_store
                    .add(&rustls_cert)
                    .map_err(|_| Error::InvalidCertificate)?;
            }
        }
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
    #[cfg_attr(docsrs, doc(cfg(any(feature = "native-tls", feature = "rustls-tls"))))]
    pub fn new(domain: String) -> Result<Self, Error> {
        TlsParametersBuilder::new(domain).build()
    }

    pub fn builder(domain: String) -> TlsParametersBuilder {
        TlsParametersBuilder::new(domain)
    }

    /// Creates a new `TlsParameters` using native-tls
    #[cfg(feature = "native-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
    pub fn new_native(domain: String) -> Result<Self, Error> {
        TlsParametersBuilder::new(domain).build_native()
    }

    /// Creates a new `TlsParameters` using rustls
    #[cfg(feature = "rustls-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rustls-tls")))]
    pub fn new_rustls(domain: String) -> Result<Self, Error> {
        TlsParametersBuilder::new(domain).build_rustls()
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }
}

/// A client certificate that can be used with [`TlsParametersBuilder::add_root_certificate`]
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct Certificate {
    #[cfg(feature = "native-tls")]
    native_tls: native_tls::Certificate,
    #[cfg(feature = "rustls-tls")]
    rustls: Vec<rustls::Certificate>,
}

impl Certificate {
    /// Create a `Certificate` from a DER encoded certificate
    pub fn from_der(der: Vec<u8>) -> Result<Self, Error> {
        #[cfg(feature = "native-tls")]
        let native_tls_cert =
            native_tls::Certificate::from_der(&der).map_err(|_| Error::InvalidCertificate)?;

        Ok(Self {
            #[cfg(feature = "native-tls")]
            native_tls: native_tls_cert,
            #[cfg(feature = "rustls-tls")]
            rustls: vec![rustls::Certificate(der)],
        })
    }

    /// Create a `Certificate` from a PEM encoded certificate
    pub fn from_pem(pem: &[u8]) -> Result<Self, Error> {
        #[cfg(feature = "native-tls")]
        let native_tls_cert =
            native_tls::Certificate::from_pem(pem).map_err(|_| Error::InvalidCertificate)?;

        #[cfg(feature = "rustls-tls")]
        let rustls_cert = {
            use rustls::internal::pemfile;
            use std::io::Cursor;

            let mut pem = Cursor::new(pem);
            pemfile::certs(&mut pem).map_err(|_| Error::InvalidCertificate)?
        };

        Ok(Self {
            #[cfg(feature = "native-tls")]
            native_tls: native_tls_cert,
            #[cfg(feature = "rustls-tls")]
            rustls: rustls_cert,
        })
    }
}

#[cfg(feature = "rustls-tls")]
struct InvalidCertsVerifier;

#[cfg(feature = "rustls-tls")]
impl ServerCertVerifier for InvalidCertsVerifier {
    fn verify_server_cert(
        &self,
        _roots: &RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: DNSNameRef<'_>,
        _ocsp_response: &[u8],
    ) -> Result<ServerCertVerified, TLSError> {
        Ok(ServerCertVerified::assertion())
    }
}
