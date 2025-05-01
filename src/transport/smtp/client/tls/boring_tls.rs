use std::fmt::{self, Debug};

use boring::{
    ssl::{SslConnector, SslMethod, SslVerifyMode, SslVersion},
    x509::store::X509StoreBuilder,
};

use crate::transport::smtp::error::{self, Error};

pub(super) fn build_connector(
    builder: super::TlsParametersBuilder<super::BoringTls>,
) -> Result<SslConnector, Error> {
    let mut tls_builder = SslConnector::builder(SslMethod::tls_client()).map_err(error::tls)?;

    if builder.accept_invalid_certs {
        tls_builder.set_verify(SslVerifyMode::NONE);
    } else {
        match builder.cert_store {
            CertificateStore::System => {}
            CertificateStore::None => {
                // Replace the default store with an empty store.
                tls_builder.set_cert_store(X509StoreBuilder::new().map_err(error::tls)?.build());
            }
        }

        let cert_store = tls_builder.cert_store_mut();

        for cert in builder.root_certs {
            cert_store.add_cert(cert.0).map_err(error::tls)?;
        }
    }

    if let Some(identity) = builder.identity {
        tls_builder
            .set_certificate(identity.chain.as_ref())
            .map_err(error::tls)?;
        tls_builder
            .set_private_key(identity.key.as_ref())
            .map_err(error::tls)?;
    }

    let min_tls_version = match builder.min_tls_version {
        MinTlsVersion::Tlsv10 => SslVersion::TLS1,
        MinTlsVersion::Tlsv11 => SslVersion::TLS1_1,
        MinTlsVersion::Tlsv12 => SslVersion::TLS1_2,
        MinTlsVersion::Tlsv13 => SslVersion::TLS1_3,
    };

    tls_builder
        .set_min_proto_version(Some(min_tls_version))
        .map_err(error::tls)?;
    Ok(tls_builder.build())
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub enum CertificateStore {
    #[default]
    System,
    None,
}

#[derive(Clone)]
pub struct Certificate(pub(super) boring::x509::X509);

impl Certificate {
    pub fn from_pem(pem: &[u8]) -> Result<Self, Error> {
        Ok(Self(boring::x509::X509::from_pem(pem).map_err(error::tls)?))
    }

    pub fn from_der(der: &[u8]) -> Result<Self, Error> {
        Ok(Self(boring::x509::X509::from_der(der).map_err(error::tls)?))
    }
}

impl Debug for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Certificate").finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct Identity {
    pub(super) chain: boring::x509::X509,
    pub(super) key: boring::pkey::PKey<boring::pkey::Private>,
}

impl Identity {
    pub fn from_pem(pem: &[u8], key: &[u8]) -> Result<Self, Error> {
        let chain = boring::x509::X509::from_pem(pem).map_err(error::tls)?;
        let key = boring::pkey::PKey::private_key_from_pem(key).map_err(error::tls)?;
        Ok(Self { chain, key })
    }
}

impl Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identity").finish_non_exhaustive()
    }
}

#[derive(Debug, Copy, Clone, Default)]
#[non_exhaustive]
pub enum MinTlsVersion {
    Tlsv10,
    Tlsv11,
    #[default]
    Tlsv12,
    Tlsv13,
}
