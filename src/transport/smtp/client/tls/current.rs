use std::fmt::{self, Debug};

#[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
use crate::transport::smtp::{error, Error};

/// TLS protocol versions.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
#[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
pub enum TlsVersion {
    /// TLS 1.0
    ///
    /// Should only be used when trying to support legacy
    /// SMTP servers that haven't updated to
    /// at least TLS 1.2 yet.
    ///
    /// Supported by `native-tls` and `boring-tls`.
    Tlsv10,
    /// TLS 1.1
    ///
    /// Should only be used when trying to support legacy
    /// SMTP servers that haven't updated to
    /// at least TLS 1.2 yet.
    ///
    /// Supported by `native-tls` and `boring-tls`.
    Tlsv11,
    /// TLS 1.2
    ///
    /// A good option for most SMTP servers.
    ///
    /// Supported by all TLS backends.
    Tlsv12,
    /// TLS 1.3
    ///
    /// The most secure option, although not supported by all SMTP servers.
    ///
    /// Although it is technically supported by all TLS backends,
    /// trying to set it for `native-tls` will give a runtime error.
    Tlsv13,
}

/// Specifies how to establish a TLS connection
///
/// TLDR: Use [`Tls::Wrapper`] or [`Tls::Required`] when
/// connecting to a remote server, [`Tls::None`] when
/// connecting to a local server.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub enum Tls {
    /// Insecure (plaintext) connection only.
    ///
    /// This option **always** uses a plaintext connection and should only
    /// be used for trusted local relays. It is **highly discouraged**
    /// for remote servers, as it exposes credentials and emails to potential
    /// interception.
    ///
    /// Note: Servers requiring credentials or emails to be sent over TLS
    /// may reject connections when this option is used.
    None,
    /// Begin with a plaintext connection and attempt to use `STARTTLS` if available.
    ///
    /// lettre will try to upgrade to a TLS-secured connection but will fall back
    /// to plaintext if the server does not support TLS. This option is provided for
    /// compatibility but is **strongly discouraged**, as it exposes connections to
    /// potential MITM (man-in-the-middle) attacks.
    ///
    /// Warning: A malicious intermediary could intercept the `STARTTLS` flag,
    /// causing lettre to believe the server only supports plaintext connections.
    #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls")))
    )]
    Opportunistic(TlsParameters),
    /// Begin with a plaintext connection and require `STARTTLS` for security.
    ///
    /// lettre will upgrade plaintext TCP connections to TLS before transmitting
    /// any sensitive data. If the server does not support TLS, the connection
    /// attempt will fail, ensuring no credentials or emails are sent in plaintext.
    ///
    /// Unlike [`Tls::Opportunistic`], this option is secure against MITM attacks.
    /// For optimal security and performance, consider using [`Tls::Wrapper`] instead,
    /// as it requires fewer roundtrips to establish a secure connection.
    #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls")))
    )]
    Required(TlsParameters),
    /// Establish a connection wrapped in TLS from the start.
    ///
    /// lettre connects to the server and immediately performs a TLS handshake.
    /// If the handshake fails, the connection attempt is aborted without
    /// transmitting any sensitive data.
    ///
    /// This is the fastest and most secure option for establishing a connection.
    #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls")))
    )]
    Wrapper(TlsParameters),
}

impl Debug for Tls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::None => f.pad("None"),
            #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
            Self::Opportunistic(_) => f.pad("Opportunistic"),
            #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
            Self::Required(_) => f.pad("Required"),
            #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
            Self::Wrapper(_) => f.pad("Wrapper"),
        }
    }
}

/// Source for the base set of root certificates to trust.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, Default)]
pub enum CertificateStore {
    /// Use the default for the TLS backend.
    ///
    /// For native-tls, this will use the system certificate store on Windows, the keychain on
    /// macOS, and OpenSSL directories on Linux (usually `/etc/ssl`).
    ///
    /// For rustls, this will also use the system store if the `rustls-native-certs` feature is
    /// enabled, or will fall back to `webpki-roots`.
    ///
    /// The boring-tls backend uses the same logic as OpenSSL on all platforms.
    #[default]
    Default,
    /// Use a hardcoded set of Mozilla roots via the `webpki-roots` crate.
    ///
    /// This option is only available in the rustls backend.
    #[cfg(all(feature = "rustls", feature = "webpki-roots"))]
    WebpkiRoots,
    /// Don't use any system certificates.
    None,
}

/// Parameters to use for secure clients
#[derive(Clone)]
pub struct TlsParameters {
    pub(crate) connector: InnerTlsParameters,
    /// The domain name which is expected in the TLS certificate from the server
    pub(super) domain: String,
}

/// Builder for `TlsParameters`
#[derive(Debug, Clone)]
pub struct TlsParametersBuilder {
    domain: String,
    cert_store: CertificateStore,
    root_certs: Vec<Certificate>,
    identity: Option<Identity>,
    accept_invalid_hostnames: bool,
    accept_invalid_certs: bool,
    #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
    min_tls_version: TlsVersion,
}

impl TlsParametersBuilder {
    /// Creates a new builder for `TlsParameters`
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            cert_store: CertificateStore::Default,
            root_certs: Vec::new(),
            identity: None,
            accept_invalid_hostnames: false,
            accept_invalid_certs: false,
            #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
            min_tls_version: TlsVersion::Tlsv12,
        }
    }

    /// Set the source for the base set of root certificates to trust.
    pub fn certificate_store(mut self, cert_store: CertificateStore) -> Self {
        self.cert_store = cert_store;
        self
    }

    /// Add a custom root certificate
    ///
    /// Can be used to safely connect to a server using a self-signed certificate, for example.
    pub fn add_root_certificate(mut self, cert: Certificate) -> Self {
        self.root_certs.push(cert);
        self
    }

    /// Add a client certificate
    ///
    /// Can be used to configure a client certificate to present to the server.
    pub fn identify_with(mut self, identity: Identity) -> Self {
        self.identity = Some(identity);
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
    #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls")))
    )]
    pub fn dangerous_accept_invalid_hostnames(mut self, accept_invalid_hostnames: bool) -> Self {
        self.accept_invalid_hostnames = accept_invalid_hostnames;
        self
    }

    /// Controls which minimum TLS version is allowed
    ///
    /// Defaults to [`Tlsv12`][TlsVersion::Tlsv12].
    #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls")))
    )]
    pub fn set_min_tls_version(mut self, min_tls_version: TlsVersion) -> Self {
        self.min_tls_version = min_tls_version;
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

    /// Creates a new `TlsParameters` using native-tls, boring-tls or rustls
    /// depending on which one is available
    #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls")))
    )]
    pub fn build(self) -> Result<TlsParameters, Error> {
        #[cfg(feature = "rustls")]
        return self.build_rustls();
        #[cfg(all(not(feature = "rustls"), feature = "native-tls"))]
        return self.build_native();
        #[cfg(all(not(feature = "rustls"), feature = "boring-tls"))]
        return self.build_boring();
    }

    /// Creates a new `TlsParameters` using native-tls with the provided configuration
    #[cfg(feature = "native-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
    pub fn build_native(self) -> Result<TlsParameters, Error> {
        let cert_store = match self.cert_store {
            CertificateStore::Default => super::native_tls::CertificateStore::System,
            CertificateStore::None => super::native_tls::CertificateStore::None,
            #[allow(unreachable_patterns)]
            other => {
                return Err(error::tls(format!(
                    "{other:?} is not supported in native tls"
                )))
            }
        };
        let min_tls_version = match self.min_tls_version {
            TlsVersion::Tlsv10 => super::native_tls::MinTlsVersion::Tlsv10,
            TlsVersion::Tlsv11 => super::native_tls::MinTlsVersion::Tlsv11,
            TlsVersion::Tlsv12 => super::native_tls::MinTlsVersion::Tlsv12,
            TlsVersion::Tlsv13 => {
                return Err(error::tls(
                    "min tls version Tlsv13 not supported in native tls",
                ))
            }
        };

        let mut builder = super::TlsParametersBuilder::<super::NativeTls>::new(self.domain)
            .certificate_store(cert_store)
            .dangerous_accept_invalid_certs(self.accept_invalid_certs)
            .dangerous_accept_invalid_hostnames(self.accept_invalid_hostnames)
            .min_tls_version(min_tls_version);
        for cert in self.root_certs {
            builder = builder.add_root_certificate(cert.native_tls);
        }
        if let Some(identity) = self.identity {
            builder = builder.identify_with(identity.native_tls);
        }

        builder.build_legacy()
    }

    /// Creates a new `TlsParameters` using boring-tls with the provided configuration
    ///
    /// Warning: this uses the certificate store passed via `certificate_store`
    /// instead of the one configured in [`TlsParametersBuilder::certificate_store`].
    #[cfg(feature = "boring-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
    pub fn build_boring(self) -> Result<TlsParameters, Error> {
        let cert_store = match self.cert_store {
            CertificateStore::Default => super::boring_tls::CertificateStore::System,
            CertificateStore::None => super::boring_tls::CertificateStore::None,
            #[allow(unreachable_patterns)]
            other => {
                return Err(error::tls(format!(
                    "{other:?} is not supported in native tls"
                )))
            }
        };
        let min_tls_version = match self.min_tls_version {
            TlsVersion::Tlsv10 => super::boring_tls::MinTlsVersion::Tlsv10,
            TlsVersion::Tlsv11 => super::boring_tls::MinTlsVersion::Tlsv11,
            TlsVersion::Tlsv12 => super::boring_tls::MinTlsVersion::Tlsv12,
            TlsVersion::Tlsv13 => super::boring_tls::MinTlsVersion::Tlsv13,
        };

        let mut builder = super::TlsParametersBuilder::<super::BoringTls>::new(self.domain)
            .certificate_store(cert_store)
            .dangerous_accept_invalid_certs(self.accept_invalid_certs)
            .dangerous_accept_invalid_hostnames(self.accept_invalid_hostnames)
            .min_tls_version(min_tls_version);
        for cert in self.root_certs {
            builder = builder.add_root_certificate(cert.boring_tls);
        }
        if let Some(identity) = self.identity {
            builder = builder.identify_with(identity.boring_tls);
        }

        builder.build_legacy()
    }

    /// Creates a new `TlsParameters` using rustls with the provided configuration
    #[cfg(feature = "rustls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
    pub fn build_rustls(self) -> Result<TlsParameters, Error> {
        let cert_store = match self.cert_store {
            #[cfg(feature = "rustls-native-certs")]
            CertificateStore::Default => super::rustls::CertificateStore::NativeCerts,
            #[cfg(all(not(feature = "rustls-native-certs"), feature = "webpki-roots"))]
            CertificateStore::Default => super::rustls::CertificateStore::WebpkiRoots,
            #[cfg(feature = "webpki-roots")]
            CertificateStore::WebpkiRoots => super::rustls::CertificateStore::WebpkiRoots,
            CertificateStore::None => super::rustls::CertificateStore::None,
        };
        let min_tls_version = match self.min_tls_version {
            TlsVersion::Tlsv10 => {
                return Err(error::tls("min tls version Tlsv10 not supported in rustls"))
            }
            TlsVersion::Tlsv11 => {
                return Err(error::tls("min tls version Tlsv11 not supported in rustls"))
            }
            TlsVersion::Tlsv12 => super::rustls::MinTlsVersion::Tlsv12,
            TlsVersion::Tlsv13 => super::rustls::MinTlsVersion::Tlsv13,
        };

        let mut builder = super::TlsParametersBuilder::<super::Rustls>::new(self.domain)
            .certificate_store(cert_store)
            .dangerous_accept_invalid_certs(self.accept_invalid_certs)
            .dangerous_accept_invalid_hostnames(self.accept_invalid_hostnames)
            .min_tls_version(min_tls_version);
        for cert in self.root_certs {
            for cert in cert.rustls {
                builder = builder.add_root_certificate(cert);
            }
        }
        if let Some(identity) = self.identity {
            builder = builder.identify_with(identity.rustls_tls);
        }

        builder.build_legacy()
    }
}

#[derive(Clone)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum InnerTlsParameters {
    #[cfg(feature = "native-tls")]
    NativeTls { connector: native_tls::TlsConnector },
    #[cfg(feature = "rustls")]
    Rustls {
        config: std::sync::Arc<rustls::ClientConfig>,
    },
    #[cfg(feature = "boring-tls")]
    BoringTls {
        connector: boring::ssl::SslConnector,
        accept_invalid_hostnames: bool,
    },
}

impl TlsParameters {
    /// Creates a new `TlsParameters` using native-tls or rustls
    /// depending on which one is available
    #[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls")))
    )]
    pub fn new(domain: String) -> Result<Self, Error> {
        TlsParametersBuilder::new(domain).build()
    }

    /// Creates a new `TlsParameters` builder
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
    #[cfg(feature = "rustls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
    pub fn new_rustls(domain: String) -> Result<Self, Error> {
        TlsParametersBuilder::new(domain).build_rustls()
    }

    /// Creates a new `TlsParameters` using boring
    #[cfg(feature = "boring-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
    pub fn new_boring(domain: String) -> Result<Self, Error> {
        TlsParametersBuilder::new(domain).build_boring()
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }
}

/// A certificate that can be used with [`TlsParametersBuilder::add_root_certificate`]
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct Certificate {
    #[cfg(feature = "native-tls")]
    native_tls: super::native_tls::Certificate,
    #[cfg(feature = "rustls")]
    rustls: Vec<super::rustls::Certificate>,
    #[cfg(feature = "boring-tls")]
    boring_tls: super::boring_tls::Certificate,
}

#[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
impl Certificate {
    /// Create a `Certificate` from a DER encoded certificate
    pub fn from_der(der: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            #[cfg(feature = "native-tls")]
            native_tls: super::native_tls::Certificate::from_der(&der)?,
            #[cfg(feature = "boring-tls")]
            boring_tls: super::boring_tls::Certificate::from_der(&der)?,
            #[cfg(feature = "rustls")]
            rustls: vec![super::rustls::Certificate::from_der(der)?],
        })
    }

    /// Create a `Certificate` from a PEM encoded certificate
    pub fn from_pem(pem: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            #[cfg(feature = "native-tls")]
            native_tls: super::native_tls::Certificate::from_pem(pem)?,
            #[cfg(feature = "rustls")]
            rustls: super::rustls::Certificate::from_pem_bundle(pem)?,
            #[cfg(feature = "boring-tls")]
            boring_tls: super::boring_tls::Certificate::from_pem(pem)?,
        })
    }
}

impl Debug for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Certificate").finish()
    }
}

/// An identity that can be used with [`TlsParametersBuilder::identify_with`]
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct Identity {
    #[cfg(feature = "native-tls")]
    native_tls: super::native_tls::Identity,
    #[cfg(feature = "rustls")]
    rustls_tls: super::rustls::Identity,
    #[cfg(feature = "boring-tls")]
    boring_tls: super::boring_tls::Identity,
}

impl Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identity").finish()
    }
}

#[cfg(any(feature = "native-tls", feature = "rustls", feature = "boring-tls"))]
impl Identity {
    pub fn from_pem(pem: &[u8], key: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            #[cfg(feature = "native-tls")]
            native_tls: super::native_tls::Identity::from_pem(pem, key)?,
            #[cfg(feature = "rustls")]
            rustls_tls: super::rustls::Identity::from_pem(pem, key)?,
            #[cfg(feature = "boring-tls")]
            boring_tls: super::boring_tls::Identity::from_pem(pem, key)?,
        })
    }
}
