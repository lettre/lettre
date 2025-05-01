use std::fmt::{self, Debug};

use crate::transport::smtp::error::{self, Error};

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
    pub(super) key: PKey<boring::pkey::Private>,
}

impl Identity {
    pub fn from_pem(pem: &[u8], key: &[u8]) -> Result<Self, Error> {
        let cert = boring::x509::X509::from_pem(pem).map_err(error::tls)?;
        let key = boring::pkey::PKey::private_key_from_pem(key).map_err(error::tls)?;
        Ok(Self { cert, key })
    }
}

impl Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identity").finish_non_exhaustive()
    }
}
