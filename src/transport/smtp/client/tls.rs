use std::fmt::{self, Debug};
#[cfg(feature = "rustls-tls")]
use std::{sync::Arc, time::SystemTime};

#[cfg(feature = "boring-tls")]
use boring::{
    ssl::{SslConnector, SslVersion},
    x509::store::X509StoreBuilder,
};
#[cfg(feature = "native-tls")]
use native_tls::{Protocol, TlsConnector};
#[cfg(feature = "rustls-tls")]
use rustls::{
    client::{ServerCertVerified, ServerCertVerifier, WebPkiVerifier},
    ClientConfig, Error as TlsError, RootCertStore, ServerName,
};

#[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
use crate::transport::smtp::{error, Error};

/// TLS protocol versions.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
#[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
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

/// How to apply TLS to a client connection
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub enum Tls {
    /// Insecure connection only (for testing purposes)
    None,
    /// Start with insecure connection and use `STARTTLS` when available
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls")))
    )]
    Opportunistic(TlsParameters),
    /// Start with insecure connection and require `STARTTLS`
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls")))
    )]
    Required(TlsParameters),
    /// Use TLS wrapped connection
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls")))
    )]
    Wrapper(TlsParameters),
}

impl Debug for Tls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::None => f.pad("None"),
            #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
            Self::Opportunistic(_) => f.pad("Opportunistic"),
            #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
            Self::Required(_) => f.pad("Required"),
            #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
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
    /// For rustls, this will also use the the system store if the `rustls-native-certs` feature is
    /// enabled, or will fall back to `webpki-roots`.
    ///
    /// The boring-tls backend uses the same logic as OpenSSL on all platforms.
    #[default]
    Default,
    /// Use a hardcoded set of Mozilla roots via the `webpki-roots` crate.
    ///
    /// This option is only available in the rustls backend.
    #[cfg(feature = "webpki-roots")]
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
    #[cfg(feature = "boring-tls")]
    pub(super) accept_invalid_hostnames: bool,
}

/// Builder for `TlsParameters`
#[derive(Debug, Clone)]
pub struct TlsParametersBuilder {
    domain: String,
    cert_store: CertificateStore,
    root_certs: Vec<Certificate>,
    accept_invalid_hostnames: bool,
    accept_invalid_certs: bool,
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
    min_tls_version: TlsVersion,
}

impl TlsParametersBuilder {
    /// Creates a new builder for `TlsParameters`
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            cert_store: CertificateStore::Default,
            root_certs: Vec::new(),
            accept_invalid_hostnames: false,
            accept_invalid_certs: false,
            #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
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
    #[cfg(any(feature = "native-tls", feature = "boring-tls"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "native-tls", feature = "boring-tls"))))]
    pub fn dangerous_accept_invalid_hostnames(mut self, accept_invalid_hostnames: bool) -> Self {
        self.accept_invalid_hostnames = accept_invalid_hostnames;
        self
    }

    /// Controls which minimum TLS version is allowed
    ///
    /// Defaults to [`Tlsv12`][TlsVersion::Tlsv12].
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
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
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls")))
    )]
    pub fn build(self) -> Result<TlsParameters, Error> {
        #[cfg(feature = "rustls-tls")]
        return self.build_rustls();
        #[cfg(all(not(feature = "rustls-tls"), feature = "native-tls"))]
        return self.build_native();
        #[cfg(all(not(feature = "rustls-tls"), feature = "boring-tls"))]
        return self.build_boring();
    }

    /// Creates a new `TlsParameters` using native-tls with the provided configuration
    #[cfg(feature = "native-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
    pub fn build_native(self) -> Result<TlsParameters, Error> {
        let mut tls_builder = TlsConnector::builder();

        match self.cert_store {
            CertificateStore::Default => {}
            CertificateStore::None => {
                tls_builder.disable_built_in_roots(true);
            }
            #[allow(unreachable_patterns)]
            other => {
                return Err(error::tls(format!(
                    "{other:?} is not supported in native tls"
                )))
            }
        }
        for cert in self.root_certs {
            tls_builder.add_root_certificate(cert.native_tls);
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
        let connector = tls_builder.build().map_err(error::tls)?;
        Ok(TlsParameters {
            connector: InnerTlsParameters::NativeTls(connector),
            domain: self.domain,
            #[cfg(feature = "boring-tls")]
            accept_invalid_hostnames: self.accept_invalid_hostnames,
        })
    }

    /// Creates a new `TlsParameters` using boring-tls with the provided configuration
    #[cfg(feature = "boring-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
    pub fn build_boring(self) -> Result<TlsParameters, Error> {
        use boring::ssl::{SslMethod, SslVerifyMode};

        let mut tls_builder = SslConnector::builder(SslMethod::tls_client()).map_err(error::tls)?;

        if self.accept_invalid_certs {
            tls_builder.set_verify(SslVerifyMode::NONE);
        } else {
            match self.cert_store {
                CertificateStore::Default => {}
                CertificateStore::None => {
                    // Replace the default store with an empty store.
                    tls_builder
                        .set_cert_store(X509StoreBuilder::new().map_err(error::tls)?.build());
                }
                #[allow(unreachable_patterns)]
                other => {
                    return Err(error::tls(format!(
                        "{other:?} is not supported in boring tls"
                    )))
                }
            }

            let cert_store = tls_builder.cert_store_mut();

            for cert in self.root_certs {
                cert_store.add_cert(cert.boring_tls).map_err(error::tls)?;
            }
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
            connector: InnerTlsParameters::BoringTls(connector),
            domain: self.domain,
            accept_invalid_hostnames: self.accept_invalid_hostnames,
        })
    }

    /// Creates a new `TlsParameters` using rustls with the provided configuration
    #[cfg(feature = "rustls-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rustls-tls")))]
    pub fn build_rustls(self) -> Result<TlsParameters, Error> {
        let tls = ClientConfig::builder();

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

        let tls = tls
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(supported_versions)
            .map_err(error::tls)?;

        let tls = if self.accept_invalid_certs {
            tls.with_custom_certificate_verifier(Arc::new(InvalidCertsVerifier {}))
        } else {
            let mut root_cert_store = RootCertStore::empty();

            #[cfg(feature = "rustls-native-certs")]
            fn load_native_roots(store: &mut RootCertStore) -> Result<(), Error> {
                let native_certs = rustls_native_certs::load_native_certs().map_err(error::tls)?;
                let mut valid_count = 0;
                let mut invalid_count = 0;
                for cert in native_certs {
                    match store.add(&rustls::Certificate(cert.0)) {
                        Ok(_) => valid_count += 1,
                        Err(err) => {
                            #[cfg(feature = "tracing")]
                            tracing::debug!("certificate parsing failed: {:?}", err);
                            invalid_count += 1;
                        }
                    }
                }
                #[cfg(feature = "tracing")]
                tracing::debug!(
                    "loaded platform certs with {valid_count} valid and {invalid_count} invalid certs"
                );
                Ok(())
            }

            #[cfg(feature = "webpki-roots")]
            fn load_webpki_roots(store: &mut RootCertStore) {
                store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
                    rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                }));
            }

            match self.cert_store {
                CertificateStore::Default => {
                    #[cfg(feature = "rustls-native-certs")]
                    load_native_roots(&mut root_cert_store)?;
                    #[cfg(all(not(feature = "rustls-native-certs"), feature = "webpki-roots"))]
                    load_webpki_roots(&mut root_cert_store);
                }
                #[cfg(feature = "webpki-roots")]
                CertificateStore::WebpkiRoots => {
                    load_webpki_roots(&mut root_cert_store);
                }
                CertificateStore::None => {}
            }
            for cert in self.root_certs {
                for rustls_cert in cert.rustls {
                    root_cert_store.add(&rustls_cert).map_err(error::tls)?;
                }
            }

            tls.with_custom_certificate_verifier(Arc::new(WebPkiVerifier::new(
                root_cert_store,
                None,
            )))
        };
        let tls = tls.with_no_client_auth();

        Ok(TlsParameters {
            connector: InnerTlsParameters::RustlsTls(Arc::new(tls)),
            domain: self.domain,
            #[cfg(feature = "boring-tls")]
            accept_invalid_hostnames: self.accept_invalid_hostnames,
        })
    }
}

#[derive(Clone)]
#[allow(clippy::enum_variant_names)]
pub enum InnerTlsParameters {
    #[cfg(feature = "native-tls")]
    NativeTls(TlsConnector),
    #[cfg(feature = "rustls-tls")]
    RustlsTls(Arc<ClientConfig>),
    #[cfg(feature = "boring-tls")]
    BoringTls(SslConnector),
}

impl TlsParameters {
    /// Creates a new `TlsParameters` using native-tls or rustls
    /// depending on which one is available
    #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls")))
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
    #[cfg(feature = "rustls-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rustls-tls")))]
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

/// A client certificate that can be used with [`TlsParametersBuilder::add_root_certificate`]
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct Certificate {
    #[cfg(feature = "native-tls")]
    native_tls: native_tls::Certificate,
    #[cfg(feature = "rustls-tls")]
    rustls: Vec<rustls::Certificate>,
    #[cfg(feature = "boring-tls")]
    boring_tls: boring::x509::X509,
}

#[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
impl Certificate {
    /// Create a `Certificate` from a DER encoded certificate
    pub fn from_der(der: Vec<u8>) -> Result<Self, Error> {
        #[cfg(feature = "native-tls")]
        let native_tls_cert = native_tls::Certificate::from_der(&der).map_err(error::tls)?;

        #[cfg(feature = "boring-tls")]
        let boring_tls_cert = boring::x509::X509::from_der(&der).map_err(error::tls)?;

        Ok(Self {
            #[cfg(feature = "native-tls")]
            native_tls: native_tls_cert,
            #[cfg(feature = "rustls-tls")]
            rustls: vec![rustls::Certificate(der)],
            #[cfg(feature = "boring-tls")]
            boring_tls: boring_tls_cert,
        })
    }

    /// Create a `Certificate` from a PEM encoded certificate
    pub fn from_pem(pem: &[u8]) -> Result<Self, Error> {
        #[cfg(feature = "native-tls")]
        let native_tls_cert = native_tls::Certificate::from_pem(pem).map_err(error::tls)?;

        #[cfg(feature = "boring-tls")]
        let boring_tls_cert = boring::x509::X509::from_pem(pem).map_err(error::tls)?;

        #[cfg(feature = "rustls-tls")]
        let rustls_cert = {
            use std::io::Cursor;

            let mut pem = Cursor::new(pem);
            rustls_pemfile::certs(&mut pem)
                .map_err(|_| error::tls("invalid certificates"))?
                .into_iter()
                .map(rustls::Certificate)
                .collect::<Vec<_>>()
        };

        Ok(Self {
            #[cfg(feature = "native-tls")]
            native_tls: native_tls_cert,
            #[cfg(feature = "rustls-tls")]
            rustls: rustls_cert,
            #[cfg(feature = "boring-tls")]
            boring_tls: boring_tls_cert,
        })
    }
}

impl Debug for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Certificate").finish()
    }
}

#[cfg(feature = "rustls-tls")]
struct InvalidCertsVerifier;

#[cfg(feature = "rustls-tls")]
impl ServerCertVerifier for InvalidCertsVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }
}
