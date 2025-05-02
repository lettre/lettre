use crate::transport::smtp::Error;

#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
pub(super) mod boring_tls;
pub(super) mod current;
#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
pub(super) mod native_tls;
#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
pub(super) mod rustls;

#[derive(Debug)]
struct TlsParametersBuilder<B: TlsBackend> {
    domain: String,
    cert_store: B::CertificateStore,
    root_certs: Vec<B::Certificate>,
    identity: Option<B::Identity>,
    accept_invalid_certs: bool,
    accept_invalid_hostnames: bool,
    min_tls_version: B::MinTlsVersion,
}

impl<B: TlsBackend> TlsParametersBuilder<B> {
    fn new(domain: String) -> Self {
        Self {
            domain,
            cert_store: Default::default(),
            root_certs: Vec::new(),
            identity: None,
            accept_invalid_certs: false,
            accept_invalid_hostnames: false,
            min_tls_version: Default::default(),
        }
    }

    fn certificate_store(mut self, cert_store: B::CertificateStore) -> Self {
        self.cert_store = cert_store;
        self
    }

    fn add_root_certificate(mut self, cert: B::Certificate) -> Self {
        self.root_certs.push(cert);
        self
    }

    fn identify_with(mut self, identity: B::Identity) -> Self {
        self.identity = Some(identity);
        self
    }

    fn min_tls_version(mut self, min_tls_version: B::MinTlsVersion) -> Self {
        self.min_tls_version = min_tls_version;
        self
    }

    fn dangerous_accept_invalid_certs(mut self, accept_invalid_certs: bool) -> Self {
        self.accept_invalid_certs = accept_invalid_certs;
        self
    }

    fn dangerous_accept_invalid_hostnames(mut self, accept_invalid_hostnames: bool) -> Self {
        self.accept_invalid_hostnames = accept_invalid_hostnames;
        self
    }

    fn build_legacy(self) -> Result<self::current::TlsParameters, Error> {
        let domain = self.domain.clone();
        let connector = B::__build_legacy_connector(self)?;
        Ok(self::current::TlsParameters { connector, domain })
    }
}

trait TlsBackend: private::SealedTlsBackend {
    type CertificateStore: Default;
    type Certificate;
    type Identity;
    type MinTlsVersion: Default;

    #[doc(hidden)]
    fn __build_connector(builder: TlsParametersBuilder<Self>) -> Result<Self::Connector, Error>;

    #[doc(hidden)]
    #[allow(private_interfaces)]
    fn __build_legacy_connector(
        builder: TlsParametersBuilder<Self>,
    ) -> Result<self::current::InnerTlsParameters, Error>;
}

#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
#[derive(Debug)]
#[allow(missing_copy_implementations)]
#[non_exhaustive]
pub(super) struct BoringTls;

#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
impl TlsBackend for BoringTls {
    type CertificateStore = self::boring_tls::CertificateStore;
    type Certificate = self::boring_tls::Certificate;
    type Identity = self::boring_tls::Identity;
    type MinTlsVersion = self::boring_tls::MinTlsVersion;

    #[allow(private_interfaces)]
    fn __build_connector(builder: TlsParametersBuilder<Self>) -> Result<Self::Connector, Error> {
        self::boring_tls::build_connector(builder)
    }

    #[allow(private_interfaces)]
    fn __build_legacy_connector(
        builder: TlsParametersBuilder<Self>,
    ) -> Result<self::current::InnerTlsParameters, Error> {
        let accept_invalid_hostnames = builder.accept_invalid_hostnames;
        Self::__build_connector(builder).map(|connector| {
            self::current::InnerTlsParameters::BoringTls {
                connector,
                accept_invalid_hostnames,
            }
        })
    }
}

#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
#[derive(Debug)]
#[allow(missing_copy_implementations)]
#[non_exhaustive]
pub(super) struct NativeTls;

#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
impl TlsBackend for NativeTls {
    type CertificateStore = self::native_tls::CertificateStore;
    type Certificate = self::native_tls::Certificate;
    type Identity = self::native_tls::Identity;
    type MinTlsVersion = self::native_tls::MinTlsVersion;

    #[allow(private_interfaces)]
    fn __build_connector(builder: TlsParametersBuilder<Self>) -> Result<Self::Connector, Error> {
        self::native_tls::build_connector(builder)
    }

    #[allow(private_interfaces)]
    fn __build_legacy_connector(
        builder: TlsParametersBuilder<Self>,
    ) -> Result<self::current::InnerTlsParameters, Error> {
        Self::__build_connector(builder)
            .map(|connector| self::current::InnerTlsParameters::NativeTls { connector })
    }
}

#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
#[derive(Debug)]
#[allow(missing_copy_implementations)]
#[non_exhaustive]
pub(super) struct Rustls;

#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
impl TlsBackend for Rustls {
    type CertificateStore = self::rustls::CertificateStore;
    type Certificate = self::rustls::Certificate;
    type Identity = self::rustls::Identity;
    type MinTlsVersion = self::rustls::MinTlsVersion;

    #[allow(private_interfaces)]
    fn __build_connector(builder: TlsParametersBuilder<Self>) -> Result<Self::Connector, Error> {
        self::rustls::build_connector(builder)
    }

    #[allow(private_interfaces)]
    fn __build_legacy_connector(
        builder: TlsParametersBuilder<Self>,
    ) -> Result<self::current::InnerTlsParameters, Error> {
        Self::__build_connector(builder)
            .map(|config| self::current::InnerTlsParameters::Rustls { config })
    }
}

mod private {
    pub(super) trait SealedTlsBackend: Sized {
        type Connector;
    }

    #[cfg(feature = "boring-tls")]
    impl SealedTlsBackend for super::BoringTls {
        type Connector = boring::ssl::SslConnector;
    }

    #[cfg(feature = "native-tls")]
    impl SealedTlsBackend for super::NativeTls {
        type Connector = native_tls::TlsConnector;
    }

    #[cfg(feature = "rustls")]
    impl SealedTlsBackend for super::Rustls {
        type Connector = std::sync::Arc<rustls::client::ClientConfig>;
    }
}
