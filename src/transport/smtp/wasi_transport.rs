//! WASI SMTP Transport for lettre (wasip3)
//! Provides a high-level API for sending emails over WASI sockets

#![cfg(all(target_arch = "wasm32", feature = "wasi"))]

use crate::address::Envelope;
use crate::transport::smtp::client::wasi_connection::WasiSmtpConnection;
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
}

impl WasiSmtpClient {
    pub(super) async fn connection(&self) -> Result<WasiSmtpConnection, Error> {
        let conn = WasiSmtpConnection::connect_wasi(
            &self.info.server,
            self.info.port,
            self.info.timeout,
            &self.info.hello_name,
        )
        .await?;
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
