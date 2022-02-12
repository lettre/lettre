use std::fmt::Debug;
#[cfg(feature = "smtp-transport")]
use std::future::Future;
#[cfg(feature = "file-transport")]
use std::io::Result as IoResult;
#[cfg(feature = "file-transport")]
use std::path::Path;
#[cfg(feature = "smtp-transport")]
use std::time::Duration;

use async_trait::async_trait;
#[cfg(all(feature = "smtp-transport", feature = "async-std1"))]
use futures_util::future::BoxFuture;

#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio1", feature = "async-std1")
))]
use crate::transport::smtp::client::AsyncSmtpConnection;
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio1", feature = "async-std1")
))]
use crate::transport::smtp::client::Tls;
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio1", feature = "async-std1")
))]
use crate::transport::smtp::extension::ClientId;
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "tokio1", feature = "async-std1")
))]
use crate::transport::smtp::Error;

/// Async executor abstraction trait
///
/// Used by [`AsyncSmtpTransport`], [`AsyncSendmailTransport`] and [`AsyncFileTransport`]
/// in order to be able to work with different async runtimes.
///
/// [`AsyncSmtpTransport`]: crate::AsyncSmtpTransport
/// [`AsyncSendmailTransport`]: crate::AsyncSendmailTransport
/// [`AsyncFileTransport`]: crate::AsyncFileTransport
#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio1", feature = "async-std1"))))]
#[async_trait]
pub trait Executor: Debug + Send + Sync + 'static + private::Sealed {
    #[cfg(feature = "smtp-transport")]
    type Handle: SpawnHandle;
    #[cfg(feature = "smtp-transport")]
    type Sleep: Future<Output = ()> + Send + 'static;

    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    fn spawn<F>(fut: F) -> Self::Handle
    where
        F: Future<Output = ()> + Send + 'static,
        F::Output: Send + 'static;

    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    fn sleep(duration: Duration) -> Self::Sleep;

    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    async fn connect(
        hostname: &str,
        port: u16,
        timeout: Option<Duration>,
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

#[doc(hidden)]
#[cfg(feature = "smtp-transport")]
#[async_trait]
pub trait SpawnHandle: Debug + Send + Sync + 'static + private::Sealed {
    async fn shutdown(self);
}

/// Async [`Executor`] using `tokio` `1.x`
///
/// Used by [`AsyncSmtpTransport`], [`AsyncSendmailTransport`] and [`AsyncFileTransport`]
/// in order to be able to work with different async runtimes.
///
/// [`AsyncSmtpTransport`]: crate::AsyncSmtpTransport
/// [`AsyncSendmailTransport`]: crate::AsyncSendmailTransport
/// [`AsyncFileTransport`]: crate::AsyncFileTransport
#[allow(missing_copy_implementations)]
#[non_exhaustive]
#[cfg(feature = "tokio1")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio1")))]
#[derive(Debug)]
pub struct Tokio1Executor;

#[async_trait]
#[cfg(feature = "tokio1")]
impl Executor for Tokio1Executor {
    #[cfg(feature = "smtp-transport")]
    type Handle = tokio1_crate::task::JoinHandle<()>;
    #[cfg(feature = "smtp-transport")]
    type Sleep = tokio1_crate::time::Sleep;

    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    fn spawn<F>(fut: F) -> Self::Handle
    where
        F: Future<Output = ()> + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio1_crate::spawn(fut)
    }

    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    fn sleep(duration: Duration) -> Self::Sleep {
        tokio1_crate::time::sleep(duration)
    }

    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    async fn connect(
        hostname: &str,
        port: u16,
        timeout: Option<Duration>,
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
        let mut conn = AsyncSmtpConnection::connect_tokio1(
            (hostname, port),
            timeout,
            hello_name,
            tls_parameters,
        )
        .await?;

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

#[cfg(all(feature = "smtp-transport", feature = "tokio1"))]
#[async_trait]
impl SpawnHandle for tokio1_crate::task::JoinHandle<()> {
    async fn shutdown(self) {
        self.abort();
    }
}

/// Async [`Executor`] using `async-std` `1.x`
///
/// Used by [`AsyncSmtpTransport`], [`AsyncSendmailTransport`] and [`AsyncFileTransport`]
/// in order to be able to work with different async runtimes.
///
/// [`AsyncSmtpTransport`]: crate::AsyncSmtpTransport
/// [`AsyncSendmailTransport`]: crate::AsyncSendmailTransport
/// [`AsyncFileTransport`]: crate::AsyncFileTransport
#[allow(missing_copy_implementations)]
#[non_exhaustive]
#[cfg(feature = "async-std1")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std1")))]
#[derive(Debug)]
pub struct AsyncStd1Executor;

#[async_trait]
#[cfg(feature = "async-std1")]
impl Executor for AsyncStd1Executor {
    #[cfg(feature = "smtp-transport")]
    type Handle = async_std::task::JoinHandle<()>;
    #[cfg(feature = "smtp-transport")]
    type Sleep = BoxFuture<'static, ()>;

    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    fn spawn<F>(fut: F) -> Self::Handle
    where
        F: Future<Output = ()> + Send + 'static,
        F::Output: Send + 'static,
    {
        async_std::task::spawn(fut)
    }

    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    fn sleep(duration: Duration) -> Self::Sleep {
        let fut = async move { async_std::task::sleep(duration).await };
        Box::pin(fut)
    }

    #[doc(hidden)]
    #[cfg(feature = "smtp-transport")]
    async fn connect(
        hostname: &str,
        port: u16,
        timeout: Option<Duration>,
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
        let mut conn = AsyncSmtpConnection::connect_asyncstd1(
            (hostname, port),
            timeout,
            hello_name,
            tls_parameters,
        )
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

#[cfg(all(feature = "smtp-transport", feature = "async-std1"))]
#[async_trait]
impl SpawnHandle for async_std::task::JoinHandle<()> {
    async fn shutdown(self) {
        self.cancel().await;
    }
}

mod private {
    use super::*;

    pub trait Sealed {}

    #[cfg(feature = "tokio1")]
    impl Sealed for Tokio1Executor {}

    #[cfg(feature = "async-std1")]
    impl Sealed for AsyncStd1Executor {}

    #[cfg(all(feature = "smtp-transport", feature = "tokio1"))]
    impl Sealed for tokio1_crate::task::JoinHandle<()> {}

    #[cfg(all(feature = "smtp-transport", feature = "async-std1"))]
    impl Sealed for async_std::task::JoinHandle<()> {}
}
