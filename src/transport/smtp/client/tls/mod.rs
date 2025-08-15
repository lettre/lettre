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
#[allow(private_bounds)]
pub(in crate::transport::smtp) struct TlsParameters<B: TlsBackend> {
    pub(in crate::transport::smtp) server_name: B::ServerName,
    pub(in crate::transport::smtp) connector: B::Connector,
    pub(in crate::transport::smtp) extra_info: B::ExtraInfo,
}

impl<B: TlsBackend> Clone for TlsParameters<B> {
    fn clone(&self) -> Self {
        Self {
            server_name: self.server_name.clone(),
            connector: self.connector.clone(),
            extra_info: self.extra_info.clone(),
        }
    }
}

#[derive(Debug)]
struct TlsParametersBuilder<B: TlsBackend> {
    server_name: String,
    cert_store: B::CertificateStore,
    root_certs: Vec<B::Certificate>,
    identity: Option<B::Identity>,
    accept_invalid_certs: bool,
    accept_invalid_hostnames: bool,
    min_tls_version: B::MinTlsVersion,
}

impl<B: TlsBackend> TlsParametersBuilder<B> {
    fn new(server_name: String) -> Self {
        Self {
            server_name,
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

    fn build(self) -> Result<TlsParameters<B>, Error> {
        B::__build_connector(self)
    }
}

#[allow(private_bounds)]
trait TlsBackend: private::SealedTlsBackend {
    type CertificateStore: Default;
    type Certificate;
    type Identity;
    type MinTlsVersion: Default;

    #[doc(hidden)]
    fn __build_connector(builder: TlsParametersBuilder<Self>)
        -> Result<TlsParameters<Self>, Error>;

    #[doc(hidden)]
    fn __build_current_tls_parameters(inner: TlsParameters<Self>) -> self::current::TlsParameters;
}

#[cfg(feature = "native-tls")]
type DefaultTlsBackend = NativeTls;

#[cfg(all(feature = "rustls", not(feature = "native-tls")))]
type DefaultTlsBackend = Rustls;

#[cfg(all(
    feature = "boring-tls",
    not(feature = "native-tls"),
    not(feature = "rustls")
))]
type DefaultTlsBackend = BoringTls;

#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
#[derive(Debug)]
#[allow(missing_copy_implementations)]
#[non_exhaustive]
pub(in crate::transport::smtp) struct NativeTls;

#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
impl TlsBackend for NativeTls {
    type CertificateStore = self::native_tls::CertificateStore;
    type Certificate = self::native_tls::Certificate;
    type Identity = self::native_tls::Identity;
    type MinTlsVersion = self::native_tls::MinTlsVersion;

    fn __build_connector(
        builder: TlsParametersBuilder<Self>,
    ) -> Result<TlsParameters<Self>, Error> {
        self::native_tls::build_connector(builder).map(|(server_name, connector)| TlsParameters {
            server_name,
            connector,
            extra_info: (),
        })
    }

    fn __build_current_tls_parameters(inner: TlsParameters<Self>) -> self::current::TlsParameters {
        self::current::TlsParameters {
            inner: self::current::InnerTlsParameters::NativeTls(inner),
        }
    }
}

#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
#[derive(Debug)]
#[allow(missing_copy_implementations)]
#[non_exhaustive]
pub(in crate::transport::smtp) struct Rustls;

#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
impl TlsBackend for Rustls {
    type CertificateStore = self::rustls::CertificateStore;
    type Certificate = self::rustls::Certificate;
    type Identity = self::rustls::Identity;
    type MinTlsVersion = self::rustls::MinTlsVersion;

    fn __build_connector(
        builder: TlsParametersBuilder<Self>,
    ) -> Result<TlsParameters<Self>, Error> {
        self::rustls::build_connector(builder).map(|(server_name, connector)| TlsParameters {
            server_name,
            connector,
            extra_info: (),
        })
    }

    fn __build_current_tls_parameters(inner: TlsParameters<Self>) -> self::current::TlsParameters {
        self::current::TlsParameters {
            inner: self::current::InnerTlsParameters::Rustls(inner),
        }
    }
}

#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
#[derive(Debug)]
#[allow(missing_copy_implementations)]
#[non_exhaustive]
pub(in crate::transport::smtp) struct BoringTls;

#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
impl TlsBackend for BoringTls {
    type CertificateStore = self::boring_tls::CertificateStore;
    type Certificate = self::boring_tls::Certificate;
    type Identity = self::boring_tls::Identity;
    type MinTlsVersion = self::boring_tls::MinTlsVersion;

    fn __build_connector(
        builder: TlsParametersBuilder<Self>,
    ) -> Result<TlsParameters<Self>, Error> {
        let accept_invalid_hostnames = builder.accept_invalid_hostnames;
        self::boring_tls::build_connector(builder).map(|(server_name, connector)| TlsParameters {
            server_name,
            connector,
            extra_info: BoringTlsExtraInfo {
                accept_invalid_hostnames,
            },
        })
    }

    fn __build_current_tls_parameters(inner: TlsParameters<Self>) -> self::current::TlsParameters {
        self::current::TlsParameters {
            inner: self::current::InnerTlsParameters::BoringTls(inner),
        }
    }
}

#[cfg(feature = "boring-tls")]
#[derive(Debug, Clone)]
pub(in crate::transport::smtp) struct BoringTlsExtraInfo {
    pub(super) accept_invalid_hostnames: bool,
}

mod private {
    pub(in crate::transport::smtp) trait SealedTlsBackend:
        Sized
    {
        type ServerName: Clone + AsRef<str>;
        type Connector: Clone;
        type ExtraInfo: Clone;
    }

    #[cfg(feature = "native-tls")]
    impl SealedTlsBackend for super::NativeTls {
        type ServerName = Box<str>;
        type Connector = native_tls::TlsConnector;
        type ExtraInfo = ();
    }

    #[cfg(feature = "rustls")]
    impl SealedTlsBackend for super::Rustls {
        type ServerName = super::rustls::ServerName;
        type Connector = std::sync::Arc<rustls::client::ClientConfig>;
        type ExtraInfo = ();
    }

    #[cfg(feature = "boring-tls")]
    impl SealedTlsBackend for super::BoringTls {
        type ServerName = Box<str>;
        type Connector = boring::ssl::SslConnector;
        type ExtraInfo = super::BoringTlsExtraInfo;
    }
}
