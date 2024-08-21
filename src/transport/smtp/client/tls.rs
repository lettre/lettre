use std::fmt::{self, Debug};
#[cfg(feature = "rustls-tls")]
use std::{io, sync::Arc};

#[cfg(feature = "boring-tls")]
use boring::{
    pkey::PKey,
    ssl::{SslConnector, SslVersion},
    x509::store::X509StoreBuilder,
};
#[cfg(feature = "native-tls")]
use native_tls::{Protocol, TlsConnector};
#[cfg(feature = "rustls-tls")]
use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::WebPkiSupportedAlgorithms,
    crypto::{verify_tls12_signature, verify_tls13_signature},
    pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime},
    server::ParsedCertificate,
    ClientConfig, DigitallySignedStruct, Error as TlsError, RootCertStore, SignatureScheme,
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
    /// For rustls, this will also use the system store if the `rustls-native-certs` feature is
    /// enabled, or will fall back to `webpki-roots`.
    ///
    /// The boring-tls backend uses the same logic as OpenSSL on all platforms.
    #[default]
    Default,
    /// Use a hardcoded set of Mozilla roots via the `webpki-roots` crate.
    ///
    /// This option is only available in the rustls backend.
    #[cfg(feature = "rustls-tls")]
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
    identity: Option<Identity>,
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
            identity: None,
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
    #[cfg(any(feature = "native-tls", feature = "boring-tls"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls")))
    )]
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
        if let Some(identity) = self.identity {
            tls_builder.identity(identity.native_tls);
        }

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

        if let Some(identity) = self.identity {
            tls_builder
                .set_certificate(identity.boring_tls.0.as_ref())
                .map_err(error::tls)?;
            tls_builder
                .set_private_key(identity.boring_tls.1.as_ref())
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
            connector: InnerTlsParameters::BoringTls(connector),
            domain: self.domain,
            accept_invalid_hostnames: self.accept_invalid_hostnames,
        })
    }

    /// Creates a new `TlsParameters` using rustls with the provided configuration
    #[cfg(feature = "rustls-tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rustls-tls")))]
    pub fn build_rustls(self) -> Result<TlsParameters, Error> {
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

        let tls = ClientConfig::builder_with_protocol_versions(supported_versions);
        let provider = rustls::crypto::CryptoProvider::get_default()
            .map(|arc| arc.clone())
            .unwrap_or_else(|| Arc::new(rustls::crypto::ring::default_provider()));

        // Build TLS config
        let signature_algorithms = provider.signature_verification_algorithms;

        let mut root_cert_store = RootCertStore::empty();

        #[cfg(feature = "rustls-native-certs")]
        fn load_native_roots(store: &mut RootCertStore) -> Result<(), Error> {
            let native_certs = rustls_native_certs::load_native_certs().map_err(error::tls)?;
            let (added, ignored) = store.add_parsable_certificates(native_certs);
            #[cfg(feature = "tracing")]
            tracing::debug!(
                "loaded platform certs with {added} valid and {ignored} ignored (invalid) certs"
            );
            Ok(())
        }

        #[cfg(feature = "rustls-tls")]
        fn load_webpki_roots(store: &mut RootCertStore) {
            store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        }

        match self.cert_store {
            CertificateStore::Default => {
                #[cfg(feature = "rustls-native-certs")]
                load_native_roots(&mut root_cert_store)?;
                #[cfg(not(feature = "rustls-native-certs"))]
                load_webpki_roots(&mut root_cert_store);
            }
            #[cfg(feature = "rustls-tls")]
            CertificateStore::WebpkiRoots => {
                load_webpki_roots(&mut root_cert_store);
            }
            CertificateStore::None => {}
        }
        for cert in self.root_certs {
            for rustls_cert in cert.rustls {
                root_cert_store.add(rustls_cert).map_err(error::tls)?;
            }
        }

        let tls = if self.accept_invalid_certs || self.accept_invalid_hostnames {
            let verifier = InvalidCertsVerifier {
                ignore_invalid_hostnames: self.accept_invalid_hostnames,
                ignore_invalid_certs: self.accept_invalid_certs,
                roots: root_cert_store,
                signature_algorithms,
            };
            tls.dangerous()
                .with_custom_certificate_verifier(Arc::new(verifier))
        } else {
            tls.with_root_certificates(root_cert_store)
        };

        let tls = if let Some(identity) = self.identity {
            let (client_certificates, private_key) = identity.rustls_tls;
            tls.with_client_auth_cert(client_certificates, private_key)
                .map_err(error::tls)?
        } else {
            tls.with_no_client_auth()
        };

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

/// A certificate that can be used with [`TlsParametersBuilder::add_root_certificate`]
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct Certificate {
    #[cfg(feature = "native-tls")]
    native_tls: native_tls::Certificate,
    #[cfg(feature = "rustls-tls")]
    rustls: Vec<CertificateDer<'static>>,
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
            rustls: vec![der.into()],
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
                .collect::<io::Result<Vec<_>>>()
                .map_err(|_| error::tls("invalid certificates"))?
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

/// An identity that can be used with [`TlsParametersBuilder::identify_with`]
#[allow(missing_copy_implementations)]
pub struct Identity {
    #[cfg(feature = "native-tls")]
    native_tls: native_tls::Identity,
    #[cfg(feature = "rustls-tls")]
    rustls_tls: (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>),
    #[cfg(feature = "boring-tls")]
    boring_tls: (boring::x509::X509, PKey<boring::pkey::Private>),
}

impl Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identity").finish()
    }
}

impl Clone for Identity {
    fn clone(&self) -> Self {
        Identity {
            #[cfg(feature = "native-tls")]
            native_tls: self.native_tls.clone(),
            #[cfg(feature = "rustls-tls")]
            rustls_tls: (self.rustls_tls.0.clone(), self.rustls_tls.1.clone_key()),
            #[cfg(feature = "boring-tls")]
            boring_tls: (self.boring_tls.0.clone(), self.boring_tls.1.clone()),
        }
    }
}

#[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
impl Identity {
    pub fn from_pem(pem: &[u8], key: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            #[cfg(feature = "native-tls")]
            native_tls: Identity::from_pem_native_tls(pem, key)?,
            #[cfg(feature = "rustls-tls")]
            rustls_tls: Identity::from_pem_rustls_tls(pem, key)?,
            #[cfg(feature = "boring-tls")]
            boring_tls: Identity::from_pem_boring_tls(pem, key)?,
        })
    }

    #[cfg(feature = "native-tls")]
    fn from_pem_native_tls(pem: &[u8], key: &[u8]) -> Result<native_tls::Identity, Error> {
        native_tls::Identity::from_pkcs8(pem, key).map_err(error::tls)
    }

    #[cfg(feature = "rustls-tls")]
    fn from_pem_rustls_tls(
        pem: &[u8],
        key: &[u8],
    ) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), Error> {
        let mut key = key;
        let key = rustls_pemfile::private_key(&mut key).unwrap().unwrap();
        Ok((vec![pem.to_owned().into()], key))
    }

    #[cfg(feature = "boring-tls")]
    fn from_pem_boring_tls(
        pem: &[u8],
        key: &[u8],
    ) -> Result<(boring::x509::X509, PKey<boring::pkey::Private>), Error> {
        let cert = boring::x509::X509::from_pem(pem).map_err(error::tls)?;
        let key = boring::pkey::PKey::private_key_from_pem(key).map_err(error::tls)?;
        Ok((cert, key))
    }
}

#[cfg(feature = "rustls-tls")]
#[derive(Debug)]
struct InvalidCertsVerifier {
    ignore_invalid_hostnames: bool,
    ignore_invalid_certs: bool,
    roots: RootCertStore,
    signature_algorithms: WebPkiSupportedAlgorithms,
}

#[cfg(feature = "rustls-tls")]
impl ServerCertVerifier for InvalidCertsVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        let cert = ParsedCertificate::try_from(end_entity)?;

        if !self.ignore_invalid_certs {
            rustls::client::verify_server_cert_signed_by_trust_anchor(
                &cert,
                &self.roots,
                intermediates,
                now,
                self.signature_algorithms.all,
            )?;
        }

        if !self.ignore_invalid_hostnames {
            rustls::client::verify_server_name(&cert, server_name)?;
        }
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
