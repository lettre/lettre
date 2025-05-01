use std::fmt::{self, Debug};
#[cfg(feature = "rustls")]
use std::sync::Arc;

#[cfg(feature = "boring-tls")]
use boring::{
    ssl::{SslConnector, SslVersion},
    x509::store::X509StoreBuilder,
};
#[cfg(feature = "native-tls")]
use native_tls::{Protocol, TlsConnector};
#[cfg(feature = "rustls")]
use rustls::{ClientConfig, RootCertStore};

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
#[deprecated]
#[allow(deprecated)]
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

/// Source for the base set of root certificate to trust when using `native-tls`.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
pub enum NativeTlsCertificateStore {
    #[default]
    System,
    None,
}

/// Source for the base set of root certificate to trust when using `rustls-tls`.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone)]
#[non_exhaustive]
#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
pub enum RustlsCertificateStore {
    #[cfg(all(feature = "rustls", feature = "rustls-native-certs"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "rustls", feature = "rustls-native-certs"))))]
    NativeCerts,
    #[cfg(all(feature = "rustls", feature = "webpki-roots"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "rustls", feature = "webpki-roots"))))]
    WebpkiRoots,
    None,
}

/// Source for the base set of root certificate to trust when using `boring-tls`.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
pub enum BoringTlsCertificateStore {
    #[default]
    System,
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
    #[allow(deprecated)]
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
            #[allow(deprecated)]
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
    #[deprecated]
    #[allow(deprecated)]
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
    #[deprecated]
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
        #[allow(deprecated)]
        let certificate_store = match self.cert_store {
            CertificateStore::Default => NativeTlsCertificateStore::System,
            CertificateStore::None => NativeTlsCertificateStore::None,
            #[allow(unreachable_patterns)]
            other => {
                return Err(error::tls(format!(
                    "{other:?} is not supported in native tls"
                )))
            }
        };
        self.build_native_with_certificate_store(certificate_store)
    }

    /// Creates a new `TlsParameters` using native-tls with the provided configuration
    ///
    /// Warning: this uses the certificate store passed via `certificate_store`
    /// instead of the one configured in [`TlsParametersBuilder::certificate_store`].
    #[cfg(feature = "native-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
    pub fn build_native_with_certificate_store(
        self,
        certificate_store: NativeTlsCertificateStore,
    ) -> Result<TlsParameters, Error> {
        let mut tls_builder = TlsConnector::builder();

        match certificate_store {
            NativeTlsCertificateStore::System => {}
            NativeTlsCertificateStore::None => {
                tls_builder.disable_built_in_roots(true);
            }
        }
        for cert in self.root_certs {
            tls_builder.add_root_certificate(cert.native_tls.0);
        }
        tls_builder.danger_accept_invalid_hostnames(self.accept_invalid_hostnames);
        tls_builder.danger_accept_invalid_certs(self.accept_invalid_certs);

        let min_tls_version = match self.min_tls_version {
            TlsVersion::Tlsv10 => Protocol::Tlsv10,
            TlsVersion::Tlsv11 => Protocol::Tlsv11,
            TlsVersion::Tlsv12 => Protocol::Tlsv12,
            TlsVersion::Tlsv13 => {
                return Err(error::tls(
                    "min tls version Tlsv13 not supported in native tls",
                ))
            }
        };

        tls_builder.min_protocol_version(Some(min_tls_version));
        if let Some(identity) = self.identity {
            tls_builder.identity(identity.native_tls.0);
        }

        let connector = tls_builder.build().map_err(error::tls)?;
        Ok(TlsParameters {
            connector: InnerTlsParameters::NativeTls { connector },
            domain: self.domain,
        })
    }

    /// Creates a new `TlsParameters` using boring-tls with the provided configuration
    ///
    /// Warning: this uses the certificate store passed via `certificate_store`
    /// instead of the one configured in [`TlsParametersBuilder::certificate_store`].
    #[cfg(feature = "boring-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
    pub fn build_boring(self) -> Result<TlsParameters, Error> {
        #[allow(deprecated)]
        let certificate_store = match self.cert_store {
            CertificateStore::Default => BoringTlsCertificateStore::System,
            CertificateStore::None => BoringTlsCertificateStore::None,
            #[allow(unreachable_patterns)]
            other => {
                return Err(error::tls(format!(
                    "{other:?} is not supported in boring tls"
                )))
            }
        };
        self.build_boring_with_certificate_store(certificate_store)
    }

    /// Creates a new `TlsParameters` using boring-tls with the provided configuration
    #[cfg(feature = "boring-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
    pub fn build_boring_with_certificate_store(
        self,
        certificate_store: BoringTlsCertificateStore,
    ) -> Result<TlsParameters, Error> {
        use boring::ssl::{SslMethod, SslVerifyMode};

        let mut tls_builder = SslConnector::builder(SslMethod::tls_client()).map_err(error::tls)?;

        if self.accept_invalid_certs {
            tls_builder.set_verify(SslVerifyMode::NONE);
        } else {
            match certificate_store {
                BoringTlsCertificateStore::System => {}
                BoringTlsCertificateStore::None => {
                    // Replace the default store with an empty store.
                    tls_builder
                        .set_cert_store(X509StoreBuilder::new().map_err(error::tls)?.build());
                }
            }

            let cert_store = tls_builder.cert_store_mut();

            for cert in self.root_certs {
                cert_store.add_cert(cert.boring_tls.0).map_err(error::tls)?;
            }
        }

        if let Some(identity) = self.identity {
            tls_builder
                .set_certificate(identity.boring_tls.chain.as_ref())
                .map_err(error::tls)?;
            tls_builder
                .set_private_key(identity.boring_tls.key.as_ref())
                .map_err(error::tls)?;
        }

        let min_tls_version = match self.min_tls_version {
            TlsVersion::Tlsv10 => SslVersion::TLS1,
            TlsVersion::Tlsv11 => SslVersion::TLS1_1,
            TlsVersion::Tlsv12 => SslVersion::TLS1_2,
            TlsVersion::Tlsv13 => SslVersion::TLS1_3,
        };

        tls_builder
            .set_min_proto_version(Some(min_tls_version))
            .map_err(error::tls)?;
        let connector = tls_builder.build();
        Ok(TlsParameters {
            connector: InnerTlsParameters::BoringTls {
                connector,
                accept_invalid_hostnames: self.accept_invalid_hostnames,
            },
            domain: self.domain,
        })
    }

    /// Creates a new `TlsParameters` using rustls with the provided configuration
    #[cfg(feature = "rustls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
    pub fn build_rustls(self) -> Result<TlsParameters, Error> {
        #[allow(deprecated)]
        let certificate_store = match self.cert_store {
            #[cfg(feature = "rustls-native-certs")]
            CertificateStore::Default => RustlsCertificateStore::NativeCerts,
            #[cfg(all(not(feature = "rustls-native-certs"), feature = "webpki-roots"))]
            CertificateStore::Default => RustlsCertificateStore::WebpkiRoots,
            #[cfg(feature = "webpki-roots")]
            CertificateStore::WebpkiRoots => RustlsCertificateStore::WebpkiRoots,
            CertificateStore::None => RustlsCertificateStore::None,
        };
        self.build_rustls_with_certificate_store(certificate_store)
    }

    /// Creates a new `TlsParameters` using rustls with the provided configuration
    #[cfg(feature = "rustls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
    pub fn build_rustls_with_certificate_store(
        self,
        certificate_store: RustlsCertificateStore,
    ) -> Result<TlsParameters, Error> {
        let just_version3 = &[&rustls::version::TLS13];
        let supported_versions = match self.min_tls_version {
            TlsVersion::Tlsv10 => {
                return Err(error::tls("min tls version Tlsv10 not supported in rustls"))
            }
            TlsVersion::Tlsv11 => {
                return Err(error::tls("min tls version Tlsv11 not supported in rustls"))
            }
            TlsVersion::Tlsv12 => rustls::ALL_VERSIONS,
            TlsVersion::Tlsv13 => just_version3,
        };

        let crypto_provider = crate::rustls_crypto::crypto_provider();
        let tls = ClientConfig::builder_with_provider(Arc::clone(&crypto_provider))
            .with_protocol_versions(supported_versions)
            .map_err(error::tls)?;

        // Build TLS config
        let mut root_cert_store = RootCertStore::empty();

        match certificate_store {
            #[cfg(feature = "rustls-native-certs")]
            RustlsCertificateStore::NativeCerts => {
                let rustls_native_certs::CertificateResult { certs, errors, .. } =
                    rustls_native_certs::load_native_certs();
                let errors_len = errors.len();

                let (added, ignored) = store.add_parsable_certificates(certs);
                #[cfg(feature = "tracing")]
                tracing::debug!(
                    "loaded platform certs with {errors_len} failing to load, {added} valid and {ignored} ignored (invalid) certs"
                );
                #[cfg(not(feature = "tracing"))]
                let _ = (errors_len, added, ignored);
            }
            #[cfg(feature = "webpki-roots")]
            RustlsCertificateStore::WebpkiRoots => {
                root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            }
            RustlsCertificateStore::None => {}
        }
        for cert in self.root_certs {
            for rustls_cert in cert.rustls {
                root_cert_store.add(rustls_cert.0).map_err(error::tls)?;
            }
        }

        let tls = if self.accept_invalid_certs || self.accept_invalid_hostnames {
            let verifier = super::rustls::InvalidCertsVerifier {
                ignore_invalid_hostnames: self.accept_invalid_hostnames,
                ignore_invalid_certs: self.accept_invalid_certs,
                roots: root_cert_store,
                crypto_provider,
            };
            tls.dangerous()
                .with_custom_certificate_verifier(Arc::new(verifier))
        } else {
            tls.with_root_certificates(root_cert_store)
        };

        let tls = if let Some(identity) = self.identity {
            tls.with_client_auth_cert(identity.rustls_tls.chain, identity.rustls_tls.key)
                .map_err(error::tls)?
        } else {
            tls.with_no_client_auth()
        };

        Ok(TlsParameters {
            connector: InnerTlsParameters::Rustls {
                config: Arc::new(tls),
            },
            domain: self.domain,
        })
    }
}

#[derive(Clone)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum InnerTlsParameters {
    #[cfg(feature = "native-tls")]
    NativeTls { connector: TlsConnector },
    #[cfg(feature = "rustls")]
    Rustls { config: Arc<ClientConfig> },
    #[cfg(feature = "boring-tls")]
    BoringTls {
        connector: SslConnector,
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
        // FIXME: use something different here
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
        #[cfg(feature = "rustls")]
        let rustls_cert = {
            use rustls::pki_types::{self, pem::PemObject as _, CertificateDer};

            CertificateDer::pem_slice_iter(pem)
                .map(|cert| Ok(super::rustls::Certificate(cert?)))
                .collect::<Result<Vec<_>, pki_types::pem::Error>>()
                .map_err(|_| error::tls("invalid certificates"))?
        };

        Ok(Self {
            #[cfg(feature = "native-tls")]
            native_tls: super::native_tls::Certificate::from_pem(pem)?,
            #[cfg(feature = "rustls")]
            rustls: rustls_cert,
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
