use std::fmt::{self, Debug};

use native_tls::TlsConnector;

use crate::transport::smtp::error::{self, Error};

pub(super) fn build_connector(
    builder: super::TlsParametersBuilder<super::NativeTls>,
) -> Result<TlsConnector, Error> {
    let mut tls_builder = TlsConnector::builder();

    match builder.cert_store {
        CertificateStore::System => {}
        CertificateStore::None => {
            tls_builder.disable_built_in_roots(true);
        }
    }
    for cert in builder.root_certs {
        tls_builder.add_root_certificate(cert.0);
    }
    tls_builder.danger_accept_invalid_hostnames(builder.accept_invalid_hostnames);
    tls_builder.danger_accept_invalid_certs(builder.accept_invalid_certs);

    let min_tls_version = match builder.min_tls_version {
        MinTlsVersion::Tlsv10 => native_tls::Protocol::Tlsv10,
        MinTlsVersion::Tlsv11 => native_tls::Protocol::Tlsv11,
        MinTlsVersion::Tlsv12 => native_tls::Protocol::Tlsv12,
    };

    tls_builder.min_protocol_version(Some(min_tls_version));
    if let Some(identity) = builder.identity {
        tls_builder.identity(identity.0);
    }

    tls_builder.build().map_err(error::tls)
}

#[derive(Debug, Clone, Default)]
#[allow(missing_copy_implementations)]
#[non_exhaustive]
pub(super) enum CertificateStore {
    #[default]
    System,
    None,
}

#[derive(Clone)]
pub(super) struct Certificate(pub(super) native_tls::Certificate);

impl Certificate {
    pub(super) fn from_pem(pem: &[u8]) -> Result<Self, Error> {
        Ok(Self(
            native_tls::Certificate::from_pem(pem).map_err(error::tls)?,
        ))
    }

    pub(super) fn from_der(der: &[u8]) -> Result<Self, Error> {
        Ok(Self(
            native_tls::Certificate::from_der(der).map_err(error::tls)?,
        ))
    }
}

impl Debug for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Certificate").finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub(super) struct Identity(pub(super) native_tls::Identity);

impl Identity {
    pub(super) fn from_pem(pem: &[u8], key: &[u8]) -> Result<Self, Error> {
        Ok(Self(
            native_tls::Identity::from_pkcs8(pem, key).map_err(error::tls)?,
        ))
    }
}

impl Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identity").finish_non_exhaustive()
    }
}

#[derive(Debug, Copy, Clone, Default)]
#[non_exhaustive]
pub(super) enum MinTlsVersion {
    Tlsv10,
    Tlsv11,
    #[default]
    Tlsv12,
}
