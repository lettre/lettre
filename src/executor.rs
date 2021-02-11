use async_trait::async_trait;

#[cfg(feature = "file-transport-envelope")]
use std::io::Result as IoResult;
#[cfg(feature = "file-transport-envelope")]
use std::path::Path;

#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio02", feature = "tokio1", feature = "async-std1")
))]
use crate::transport::smtp::client::AsyncSmtpConnection;
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio02", feature = "tokio1", feature = "async-std1")
))]
use crate::transport::smtp::client::Tls;
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio02", feature = "tokio1", feature = "async-std1")
))]
use crate::transport::smtp::extension::ClientId;
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio02", feature = "tokio1", feature = "async-std1")
))]
use crate::transport::smtp::Error;

#[async_trait]
pub trait Executor: Send + Sync + private::Sealed {
    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    async fn connect(
        hostname: &str,
        port: u16,
        hello_name: &ClientId,
        tls: &Tls,
    ) -> Result<AsyncSmtpConnection, Error>;

    #[doc(hidden)]
    #[cfg(feature = "file-transport-envelope")]
    async fn fs_read(path: &Path) -> IoResult<Vec<u8>>;

    #[doc(hidden)]
    #[cfg(feature = "file-transport")]
    async fn fs_write(path: &Path, contents: &[u8]) -> IoResult<()>;
}

#[allow(missing_copy_implementations)]
#[non_exhaustive]
#[cfg(feature = "tokio02")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio02")))]
pub struct Tokio02Executor;

#[async_trait]
#[cfg(feature = "tokio02")]
impl Executor for Tokio02Executor {
    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    async fn connect(
        hostname: &str,
        port: u16,
        hello_name: &ClientId,
        tls: &Tls,
    ) -> Result<AsyncSmtpConnection, Error> {
        #[allow(clippy::match_single_binding)]
        let tls_parameters = match tls {
            #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
            Tls::Wrapper(ref tls_parameters) => Some(tls_parameters.clone()),
            _ => None,
        };
        #[allow(unused_mut)]
        let mut conn =
            AsyncSmtpConnection::connect_tokio02(hostname, port, hello_name, tls_parameters)
                .await?;

        #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
        match tls {
            Tls::Opportunistic(ref tls_parameters) => {
                if conn.can_starttls() {
                    conn.starttls(tls_parameters.clone(), hello_name).await?;
                }
            }
            Tls::Required(ref tls_parameters) => {
                conn.starttls(tls_parameters.clone(), hello_name).await?;
            }
            _ => (),
        }

        Ok(conn)
    }

    #[doc(hidden)]
    #[cfg(feature = "file-transport-envelope")]
    async fn fs_read(path: &Path) -> IoResult<Vec<u8>> {
        tokio02_crate::fs::read(path).await
    }

    #[doc(hidden)]
    #[cfg(feature = "file-transport")]
    async fn fs_write(path: &Path, contents: &[u8]) -> IoResult<()> {
        tokio02_crate::fs::write(path, contents).await
    }
}

#[allow(missing_copy_implementations)]
#[non_exhaustive]
#[cfg(feature = "tokio1")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio1")))]
pub struct Tokio1Executor;

#[async_trait]
#[cfg(feature = "tokio1")]
impl Executor for Tokio1Executor {
    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    async fn connect(
        hostname: &str,
        port: u16,
        hello_name: &ClientId,
        tls: &Tls,
    ) -> Result<AsyncSmtpConnection, Error> {
        #[allow(clippy::match_single_binding)]
        let tls_parameters = match tls {
            #[cfg(any(feature = "tokio1-native-tls", feature = "tokio1-rustls-tls"))]
            Tls::Wrapper(ref tls_parameters) => Some(tls_parameters.clone()),
            _ => None,
        };
        #[allow(unused_mut)]
        let mut conn =
            AsyncSmtpConnection::connect_tokio1(hostname, port, hello_name, tls_parameters).await?;

        #[cfg(any(feature = "tokio1-native-tls", feature = "tokio1-rustls-tls"))]
        match tls {
            Tls::Opportunistic(ref tls_parameters) => {
                if conn.can_starttls() {
                    conn.starttls(tls_parameters.clone(), hello_name).await?;
                }
            }
            Tls::Required(ref tls_parameters) => {
                conn.starttls(tls_parameters.clone(), hello_name).await?;
            }
            _ => (),
        }

        Ok(conn)
    }

    #[doc(hidden)]
    #[cfg(feature = "file-transport-envelope")]
    async fn fs_read(path: &Path) -> IoResult<Vec<u8>> {
        tokio1_crate::fs::read(path).await
    }

    #[doc(hidden)]
    #[cfg(feature = "file-transport")]
    async fn fs_write(path: &Path, contents: &[u8]) -> IoResult<()> {
        tokio1_crate::fs::write(path, contents).await
    }
}

#[allow(missing_copy_implementations)]
#[non_exhaustive]
#[cfg(feature = "async-std1")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std1")))]
pub struct AsyncStd1Executor;

#[async_trait]
#[cfg(feature = "async-std1")]
impl Executor for AsyncStd1Executor {
    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    async fn connect(
        hostname: &str,
        port: u16,
        hello_name: &ClientId,
        tls: &Tls,
    ) -> Result<AsyncSmtpConnection, Error> {
        #[allow(clippy::match_single_binding)]
        let tls_parameters = match tls {
            #[cfg(any(feature = "async-std1-native-tls", feature = "async-std1-rustls-tls"))]
            Tls::Wrapper(ref tls_parameters) => Some(tls_parameters.clone()),
            _ => None,
        };
        #[allow(unused_mut)]
        let mut conn =
            AsyncSmtpConnection::connect_asyncstd1(hostname, port, hello_name, tls_parameters)
                .await?;

        #[cfg(any(feature = "async-std1-native-tls", feature = "async-std1-rustls-tls"))]
        match tls {
            Tls::Opportunistic(ref tls_parameters) => {
                if conn.can_starttls() {
                    conn.starttls(tls_parameters.clone(), hello_name).await?;
                }
            }
            Tls::Required(ref tls_parameters) => {
                conn.starttls(tls_parameters.clone(), hello_name).await?;
            }
            _ => (),
        }

        Ok(conn)
    }

    #[doc(hidden)]
    #[cfg(feature = "file-transport-envelope")]
    async fn fs_read(path: &Path) -> IoResult<Vec<u8>> {
        async_std::fs::read(path).await
    }

    #[doc(hidden)]
    #[cfg(feature = "file-transport")]
    async fn fs_write(path: &Path, contents: &[u8]) -> IoResult<()> {
        async_std::fs::write(path, contents).await
    }
}

mod private {
    use super::*;

    pub trait Sealed {}

    #[cfg(feature = "tokio02")]
    impl Sealed for Tokio02Executor {}

    #[cfg(feature = "tokio1")]
    impl Sealed for Tokio1Executor {}

    #[cfg(feature = "async-std1")]
    impl Sealed for AsyncStd1Executor {}
}
