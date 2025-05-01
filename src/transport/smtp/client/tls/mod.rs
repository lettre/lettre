use crate::transport::smtp::Error;

#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
pub mod boring_tls;
pub(super) mod deprecated;
#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
pub mod native_tls;
#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
pub mod rustls;

#[derive(Debug)]
pub struct TlsParametersBuilder<B: TlsBackend> {
    domain: String,
    cert_store: B::CertificateStore,
    root_certs: Vec<B::Certificate>,
    identity: Option<B::Identity>,
    accept_invalid_certs: bool,
    accept_invalid_hostnames: bool,
    min_tls_version: B::MinTlsVersion,
}

impl<B: TlsBackend> TlsParametersBuilder<B> {
    pub(super) fn new(domain: String) -> Self {
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

    pub fn certificate_store(mut self, cert_store: B::CertificateStore) -> Self {
        self.cert_store = cert_store;
        self
    }

    pub fn add_root_certificate(mut self, cert: B::Certificate) -> Self {
        self.root_certs.push(cert);
        self
    }

    pub fn identify_with(mut self, identity: B::Identity) -> Self {
        self.identity = Some(identity);
        self
    }

    pub fn min_tls_version(mut self, min_tls_version: B::MinTlsVersion) -> Self {
        self.min_tls_version = min_tls_version;
        self
    }

    pub fn dangerous_accept_invalid_certs(mut self, accept_invalid_certs: bool) -> Self {
        self.accept_invalid_certs = accept_invalid_certs;
        self
    }

    pub fn dangerous_accept_invalid_hostnames(mut self, accept_invalid_hostnames: bool) -> Self {
        self.accept_invalid_hostnames = accept_invalid_hostnames;
        self
    }

    pub fn build_legacy(self) -> Result<self::deprecated::TlsParameters, Error> {
        let domain = self.domain.clone();
        let connector = B::__build_legacy_connector(self)?;
        Ok(self::deprecated::TlsParameters { connector, domain })
    }
}

pub trait TlsBackend: private::SealedTlsBackend {
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
    ) -> Result<self::deprecated::InnerTlsParameters, Error>;
}

#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
#[non_exhaustive]
pub struct BoringTls;

#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
impl TlsBackend for BoringTls {
    type CertificateStore = self::boring_tls::CertificateStore;
    type Certificate = self::boring_tls::Certificate;
    type Identity = self::boring_tls::Identity;
    type MinTlsVersion = self::boring_tls::MinTlsVersion;

    fn __build_connector(builder: TlsParametersBuilder<Self>) -> Result<Self::Connector, Error> {
        self::boring_tls::build_connector(builder)
    }

    #[allow(private_interfaces)]
    fn __build_legacy_connector(
        builder: TlsParametersBuilder<Self>,
    ) -> Result<self::deprecated::InnerTlsParameters, Error> {
        let accept_invalid_hostnames = builder.accept_invalid_hostnames;
        Self::__build_connector(builder).map(|connector| {
            self::deprecated::InnerTlsParameters::BoringTls {
                connector,
                accept_invalid_hostnames,
            }
        })
    }
}

#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
#[non_exhaustive]
pub struct NativeTls;

#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
impl TlsBackend for NativeTls {
    type CertificateStore = self::native_tls::CertificateStore;
    type Certificate = self::native_tls::Certificate;
    type Identity = self::native_tls::Identity;
    type MinTlsVersion = self::native_tls::MinTlsVersion;

    fn __build_connector(builder: TlsParametersBuilder<Self>) -> Result<Self::Connector, Error> {
        self::native_tls::build_connector(builder)
    }

    #[allow(private_interfaces)]
    fn __build_legacy_connector(
        builder: TlsParametersBuilder<Self>,
    ) -> Result<self::deprecated::InnerTlsParameters, Error> {
        Self::__build_connector(builder)
            .map(|connector| self::deprecated::InnerTlsParameters::NativeTls { connector })
    }
}

#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
#[non_exhaustive]
pub struct Rustls;

#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
impl TlsBackend for Rustls {
    type CertificateStore = self::rustls::CertificateStore;
    type Certificate = self::rustls::Certificate;
    type Identity = self::rustls::Identity;
    type MinTlsVersion = self::rustls::MinTlsVersion;

    fn __build_connector(builder: TlsParametersBuilder<Self>) -> Result<Self::Connector, Error> {
        self::rustls::build_connector(builder)
    }

    #[allow(private_interfaces)]
    fn __build_legacy_connector(
        builder: TlsParametersBuilder<Self>,
    ) -> Result<self::deprecated::InnerTlsParameters, Error> {
        Self::__build_connector(builder)
            .map(|config| self::deprecated::InnerTlsParameters::Rustls { config })
    }
}

mod private {
    // FIXME: this should be `pub(super)` but the `private_bounds` lint doesn't like it
    pub trait SealedTlsBackend: Sized {
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
