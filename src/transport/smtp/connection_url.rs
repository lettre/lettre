use std::borrow::Cow;

use url::Url;

#[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
use super::client::{Tls, TlsParameters};
#[cfg(any(feature = "tokio1", feature = "async-std1"))]
use super::AsyncSmtpTransportBuilder;
use super::{
    authentication::Credentials, error, extension::ClientId, Error, SmtpTransportBuilder,
    SMTP_PORT, SUBMISSIONS_PORT, SUBMISSION_PORT,
};

pub(crate) trait TransportBuilder {
    fn new<T: Into<String>>(server: T) -> Self;
    fn tls(self, tls: super::Tls) -> Self;
    fn port(self, port: u16) -> Self;
    fn credentials(self, credentials: Credentials) -> Self;
    fn hello_name(self, name: ClientId) -> Self;
}

impl TransportBuilder for SmtpTransportBuilder {
    fn new<T: Into<String>>(server: T) -> Self {
        Self::new(server)
    }

    fn tls(self, tls: super::Tls) -> Self {
        self.tls(tls)
    }

    fn port(self, port: u16) -> Self {
        self.port(port)
    }

    fn credentials(self, credentials: Credentials) -> Self {
        self.credentials(credentials)
    }

    fn hello_name(self, name: ClientId) -> Self {
        self.hello_name(name)
    }
}

#[cfg(any(feature = "tokio1", feature = "async-std1"))]
impl TransportBuilder for AsyncSmtpTransportBuilder {
    fn new<T: Into<String>>(server: T) -> Self {
        Self::new(server)
    }

    fn tls(self, tls: super::Tls) -> Self {
        self.tls(tls)
    }

    fn port(self, port: u16) -> Self {
        self.port(port)
    }

    fn credentials(self, credentials: Credentials) -> Self {
        self.credentials(credentials)
    }

    fn hello_name(self, name: ClientId) -> Self {
        self.hello_name(name)
    }
}

/// Create a new `SmtpTransportBuilder` or `AsyncSmtpTransportBuilder` from a connection URL
pub(crate) fn from_connection_url<B: TransportBuilder>(connection_url: &str) -> Result<B, Error> {
    let connection_url = Url::parse(connection_url).map_err(error::connection)?;
    let tls: Option<String> = connection_url
        .query_pairs()
        .find(|(k, _)| k == "tls")
        .map(|(_, v)| v.to_string());

    let host = connection_url
        .host_str()
        .ok_or_else(|| error::connection("smtp host undefined"))?;

    let mut builder = B::new(host);

    match (connection_url.scheme(), tls.as_deref()) {
        ("smtp", None) => {
            builder = builder.port(connection_url.port().unwrap_or(SMTP_PORT));
        }
        #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
        ("smtp", Some("required")) => {
            builder = builder
                .port(connection_url.port().unwrap_or(SUBMISSION_PORT))
                .tls(Tls::Required(TlsParameters::new(host.into())?));
        }
        #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
        ("smtp", Some("opportunistic")) => {
            builder = builder
                .port(connection_url.port().unwrap_or(SUBMISSION_PORT))
                .tls(Tls::Opportunistic(TlsParameters::new(host.into())?));
        }
        #[cfg(any(feature = "native-tls", feature = "rustls-tls", feature = "boring-tls"))]
        ("smtps", _) => {
            builder = builder
                .port(connection_url.port().unwrap_or(SUBMISSIONS_PORT))
                .tls(Tls::Wrapper(TlsParameters::new(host.into())?));
        }
        (scheme, tls) => {
            return Err(error::connection(format!(
                "Unknown scheme '{scheme}' or tls parameter '{tls:?}', note that a transport with TLS requires one of the TLS features"
            )))
        }
    }

    // use the path segment of the URL as name in the name in the HELO / EHLO command
    if connection_url.path().len() > 1 {
        let name = connection_url.path().trim_matches('/').to_owned();
        builder = builder.hello_name(ClientId::Domain(name));
    }

    if let Some(password) = connection_url.password() {
        let percent_decode = |s: &str| {
            percent_encoding::percent_decode_str(s)
                .decode_utf8()
                .map(Cow::into_owned)
                .map_err(error::connection)
        };
        let credentials = Credentials::new(
            percent_decode(connection_url.username())?,
            percent_decode(password)?,
        );
        builder = builder.credentials(credentials);
    }

    Ok(builder)
}
