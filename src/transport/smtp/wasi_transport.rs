//! WASI SMTP Transport for lettre (wasip3)
//! Provides a high-level API for sending emails over WASI sockets

//#![cfg(all(target_arch = "wasm32", feature = "wasi"))]

use crate::address::Envelope;
use crate::transport::smtp::authentication::{Credentials, Mechanism};
use crate::transport::smtp::client::wasi_connection::WasiSmtpConnection;
use crate::transport::smtp::client::Tls;
use crate::transport::smtp::extension::ClientId;
use crate::transport::smtp::response::Response;
use crate::transport::smtp::SmtpInfo;
use core::fmt;
use std::fmt::Debug;
use std::time::Duration;

use super::Error;

pub struct WasiSmtpTransport {
    inner: WasiSmtpClient,
}

/// Needs work for pooling
pub(super) struct WasiSmtpClient {
    info: SmtpInfo,
}

pub struct WasiSmtpTransportBuilder {
    info: SmtpInfo,
}

impl WasiSmtpTransportBuilder {
    pub(crate) fn new<T: Into<String>>(server: T) -> Self {
        let info = SmtpInfo {
            server: server.into(),
            ..Default::default()
        };
        WasiSmtpTransportBuilder { info }
    }
    pub fn hello_name(mut self, name: ClientId) -> Self {
        self.info.hello_name = name;
        self
    }
    pub fn port(mut self, port: u16) -> Self {
        self.info.port = port;
        self
    }
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.info.timeout = timeout;
        self
    }
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.info.credentials = Some(credentials);
        self
    }
    pub fn authentication(mut self, mechanisms: Vec<Mechanism>) -> Self {
        self.info.authentication = mechanisms;
        self
    }
    // #[cfg(any(
    //     feature = "wasi-tls"
    // ))]
    // #[cfg_attr(
    //     docsrs,
    //     doc(cfg(any(
    //         feature = "wasi-tls"
    //     )))
    // )]
    pub fn tls(mut self, tls: Tls) -> Self {
        self.info.tls = tls;
        self
    }
    pub fn build(self) -> WasiSmtpTransport {
        let client = WasiSmtpClient { info: self.info };
        WasiSmtpTransport { inner: client }
    }
}

impl WasiSmtpTransport {
    /// Send an email message (stub)
    pub async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Response, Error> {
        let mut conn = self.inner.connection().await?;
        let result = conn.send(envelope, email).await?;

        conn.abort().await;

        Ok(result)
    }
    pub fn unencrypted_localhost() -> WasiSmtpTransport {
        Self::builder_dangerous("localhost").build()
    }
    pub fn builder_dangerous<T: Into<String>>(server: T) -> WasiSmtpTransportBuilder {
        WasiSmtpTransportBuilder::new(server)
    }
    // #[cfg(any(feature = "wasi-tls"))]
    // #[cfg_attr(
    //     docsrs,
    //     doc(cfg(any(feature = "wasi-tls")))
    // )]
    pub fn from_url(connection_url: &str) -> Result<WasiSmtpTransportBuilder, Error> {
        super::connection_url::from_connection_url(connection_url)
    }
    pub fn relay(relay: &str) -> Result<WasiSmtpTransportBuilder, Error> {
        use super::{Tls, TlsParameters, SUBMISSIONS_PORT};

        let tls_parameters = TlsParameters::new(relay.into())?;

        Ok(Self::builder_dangerous(relay)
            .port(SUBMISSIONS_PORT)
            .tls(Tls::Wrapper(tls_parameters)))
    }
}

impl WasiSmtpClient {
    pub(super) async fn connection(&self) -> Result<WasiSmtpConnection, Error> {
        // pass the parsed tls parameters into connect_wasi
        let tls = &self.info.tls;
        let tls_parameters = match tls {
            //#[cfg(any(feature = "tokio1-native-tls", feature = "tokio1-rustls"))]
            Tls::Wrapper(tls_parameters) => Some(tls_parameters.clone()),
            _ => None,
        };
        let mut conn = WasiSmtpConnection::connect_wasi(
            &self.info.server,
            self.info.port,
            self.info.timeout,
            &self.info.hello_name,
            tls_parameters,
        )
        .await?;
        // TLS using STARTTLS : question is whether to go for implicit TLS (in WasiNetworkStream) or
        // This deprecated one ?
        // As described in [rfc3207]. Note that this mechanism has been deprecated in [rfc8314].
        //
        // [rfc3207]: https://www.rfc-editor.org/rfc/rfc3207
        // [rfc8314]: https://www.rfc-editor.org/rfc/rfc8314
        match tls {
            Tls::Opportunistic(tls_parameters) => {
                if conn.can_starttls() {
                    conn.starttls(tls_parameters.clone(), &self.info.hello_name)
                        .await?;
                }
            }
            Tls::Required(tls_parameters) => {
                conn.starttls(tls_parameters.clone(), &self.info.hello_name)
                    .await?;
            }
            _ => (),
        }
        if let Some(credentials) = &self.info.credentials {
            conn.auth(&self.info.authentication, credentials).await?;
        }
        Ok(conn)
    }
}

impl Debug for WasiSmtpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("WasiSmtpClient");
        builder.field("info", &self.info);
        builder.finish()
    }
}
