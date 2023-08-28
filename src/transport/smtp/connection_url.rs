use url::Url;

use super::{
    authentication::Credentials,
    client::{Tls, TlsParameters},
    error, Error, SmtpTransportBuilder, SMTP_PORT, SUBMISSIONS_PORT,
    SUBMISSION_PORT,
};
#[cfg(any(feature = "tokio1", feature = "async-std1"))]
use super::AsyncSmtpTransportBuilder;

pub(crate) trait TransportBuilder {
    fn new<T: Into<String>>(server: T) -> Self;
    fn tls(self, tls: super::Tls) -> Self;
    fn port(self, port: u16) -> Self;
    fn credentials(self, credentials: Credentials) -> Self;
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
}

/// Create a new SmtpTransportBuilder or AsyncSmtpTransportBuilder from a connection URL
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
        ("smtp", Some("required")) => {
            builder = builder
                .port(connection_url.port().unwrap_or(SUBMISSION_PORT))
                .tls(Tls::Required(TlsParameters::new(host.into())?))
        }
        ("smtp", Some("opportunistic")) => {
            builder = builder
                .port(connection_url.port().unwrap_or(SUBMISSION_PORT))
                .tls(Tls::Opportunistic(TlsParameters::new(host.into())?))
        }
        ("smtps", _) => {
            builder = builder
                .port(connection_url.port().unwrap_or(SUBMISSIONS_PORT))
                .tls(Tls::Wrapper(TlsParameters::new(host.into())?))
        }
        (scheme, tls) => {
            return Err(error::connection(format!(
                "unknown scheme '{scheme}' or tls parameter '{tls:?}'"
            )))
        }
    };

    if let Some(password) = connection_url.password() {
        let credentials = Credentials::new(connection_url.username().into(), password.into());
        builder = builder.credentials(credentials);
    }

    Ok(builder)
}
