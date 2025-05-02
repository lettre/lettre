use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider},
    pki_types::{self, ServerName, UnixTime},
    server::ParsedCertificate,
    ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme,
};

use crate::transport::smtp::error::{self, Error};

pub(super) fn build_connector(
    builder: super::TlsParametersBuilder<super::Rustls>,
) -> Result<Arc<ClientConfig>, Error> {
    let just_version3 = &[&rustls::version::TLS13];
    let supported_versions = match builder.min_tls_version {
        MinTlsVersion::Tlsv12 => rustls::ALL_VERSIONS,
        MinTlsVersion::Tlsv13 => just_version3,
    };

    let crypto_provider = crate::rustls_crypto::crypto_provider();
    let tls = ClientConfig::builder_with_provider(Arc::clone(&crypto_provider))
        .with_protocol_versions(supported_versions)
        .map_err(error::tls)?;

    // Build TLS config
    let mut root_cert_store = RootCertStore::empty();

    match builder.cert_store {
        #[cfg(feature = "rustls-native-certs")]
        CertificateStore::NativeCerts => {
            let rustls_native_certs::CertificateResult { certs, errors, .. } =
                rustls_native_certs::load_native_certs();
            let errors_len = errors.len();

            let (added, ignored) = store.add_parsable_certificates(certs);
            #[cfg(feature = "tracing")]
            tracing::debug!(
                "loaded platform certs with {errors_len} failing to load, {added} valid and {ignored} ignored (invalid) certs"
            );
            #[cfg(not(feature = "tracing"))]
            let _ = (errors_len, added, ignored);
        }
        #[cfg(feature = "webpki-roots")]
        CertificateStore::WebpkiRoots => {
            root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        }
        CertificateStore::None => {}
    }
    for cert in builder.root_certs {
        root_cert_store.add(cert.0).map_err(error::tls)?;
    }

    let tls = if builder.accept_invalid_certs || builder.accept_invalid_hostnames {
        let verifier = InvalidCertsVerifier {
            ignore_invalid_hostnames: builder.accept_invalid_hostnames,
            ignore_invalid_certs: builder.accept_invalid_certs,
            roots: root_cert_store,
            crypto_provider,
        };
        tls.dangerous()
            .with_custom_certificate_verifier(Arc::new(verifier))
    } else {
        tls.with_root_certificates(root_cert_store)
    };

    let tls = if let Some(identity) = builder.identity {
        tls.with_client_auth_cert(identity.chain, identity.key)
            .map_err(error::tls)?
    } else {
        tls.with_no_client_auth()
    };
    Ok(Arc::new(tls))
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub enum CertificateStore {
    #[cfg(feature = "rustls-native-certs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rustls-native-certs")))]
    #[cfg_attr(feature = "rustls-native-certs", default)]
    NativeCerts,
    #[cfg(feature = "webpki-roots")]
    #[cfg_attr(docsrs, doc(cfg(feature = "webpki-roots")))]
    #[cfg_attr(
        all(feature = "webpki-roots", not(feature = "rustls-native-certs")),
        default
    )]
    WebpkiRoots,
    #[cfg_attr(
        all(not(feature = "webpki-roots"), not(feature = "rustls-native-certs")),
        default
    )]
    None,
}

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

#[derive(Debug, Copy, Clone, Default)]
#[non_exhaustive]
pub enum MinTlsVersion {
    #[default]
    Tlsv12,
    Tlsv13,
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
