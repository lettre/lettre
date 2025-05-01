use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider},
    pki_types::{self, ServerName, UnixTime},
    server::ParsedCertificate,
    DigitallySignedStruct, RootCertStore, SignatureScheme,
};

use crate::transport::smtp::error::{self, Error};

#[derive(Clone)]
pub struct Certificate(pub(super) pki_types::CertificateDer<'static>);

impl Certificate {
    pub fn from_pem(pem: &[u8]) -> Result<Self, Error> {
        use rustls::pki_types::pem::PemObject as _;

        Ok(Self(
            pki_types::CertificateDer::from_pem_slice(pem)
                .map_err(|_| error::tls("invalid certificate"))?,
        ))
    }

    pub fn from_der(der: Vec<u8>) -> Result<Self, Error> {
        Ok(Self(der.into()))
    }
}

impl Debug for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Certificate").finish_non_exhaustive()
    }
}

pub struct Identity {
    pub(super) chain: Vec<pki_types::CertificateDer<'static>>,
    pub(super) key: pki_types::PrivateKeyDer<'static>,
}

impl Identity {
    pub fn from_pem(pem: &[u8], key: &[u8]) -> Result<Self, Error> {
        use rustls::pki_types::pem::PemObject as _;

        let key = match pki_types::PrivateKeyDer::from_pem_slice(key) {
            Ok(key) => key,
            Err(pki_types::pem::Error::NoItemsFound) => {
                return Err(error::tls("no private key found"))
            }
            Err(err) => return Err(error::tls(err)),
        };

        Ok(Self {
            chain: vec![pem.to_owned().into()],
            key,
        })
    }
}

impl Clone for Identity {
    fn clone(&self) -> Self {
        Self {
            chain: self.chain.clone(),
            key: self.key.clone_key(),
        }
    }
}

impl Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identity").finish_non_exhaustive()
    }
}

// FIXME: remove `pub(super)`
#[derive(Debug)]
pub(super) struct InvalidCertsVerifier {
    pub(super) ignore_invalid_hostnames: bool,
    pub(super) ignore_invalid_certs: bool,
    pub(super) roots: RootCertStore,
    pub(super) crypto_provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for InvalidCertsVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &pki_types::CertificateDer<'_>,
        intermediates: &[pki_types::CertificateDer<'_>],
        server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let cert = ParsedCertificate::try_from(end_entity)?;

        if !self.ignore_invalid_certs {
            rustls::client::verify_server_cert_signed_by_trust_anchor(
                &cert,
                &self.roots,
                intermediates,
                now,
                self.crypto_provider.signature_verification_algorithms.all,
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
        cert: &pki_types::CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.crypto_provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &pki_types::CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.crypto_provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.crypto_provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}
