use std::fmt::{self, Debug};

use crate::transport::smtp::error::{self, Error};

#[derive(Clone)]
pub struct Certificate(pub(super) native_tls::Certificate);

impl Certificate {
    pub fn from_pem(pem: &[u8]) -> Result<Self, Error> {
        Ok(Self(
            native_tls::Certificate::from_pem(pem).map_err(error::tls)?,
        ))
    }

    pub fn from_der(der: &[u8]) -> Result<Self, Error> {
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
pub struct Identity(pub(super) native_tls::Identity);

impl Identity {
    pub fn from_pem(pem: &[u8], key: &[u8]) -> Result<Self, Error> {
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
