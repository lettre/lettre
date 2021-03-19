#[cfg(any(
    feature = "tokio02-rustls-tls",
    feature = "tokio1-rustls-tls",
    feature = "async-std1-rustls-tls"
))]
use std::sync::Arc;
use std::{
    mem,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use futures_io::{
    AsyncRead as FuturesAsyncRead, AsyncWrite as FuturesAsyncWrite, Error as IoError, ErrorKind,
    Result as IoResult,
};
#[cfg(feature = "tokio02")]
use tokio02_crate::io::{AsyncRead as _, AsyncWrite as _};
#[cfg(feature = "tokio1")]
use tokio1_crate::io::{AsyncRead as _, AsyncWrite as _, ReadBuf as Tokio1ReadBuf};

#[cfg(feature = "async-std1")]
use async_std::net::TcpStream as AsyncStd1TcpStream;
#[cfg(feature = "tokio02")]
use tokio02_crate::net::TcpStream as Tokio02TcpStream;
#[cfg(feature = "tokio1")]
use tokio1_crate::net::TcpStream as Tokio1TcpStream;

#[cfg(feature = "async-std1-native-tls")]
use async_native_tls::TlsStream as AsyncStd1TlsStream;
#[cfg(feature = "tokio02-native-tls")]
use tokio02_native_tls_crate::TlsStream as Tokio02TlsStream;
#[cfg(feature = "tokio1-native-tls")]
use tokio1_native_tls_crate::TlsStream as Tokio1TlsStream;

#[cfg(feature = "async-std1-rustls-tls")]
use async_rustls::client::TlsStream as AsyncStd1RustlsTlsStream;
#[cfg(feature = "tokio02-rustls-tls")]
use tokio02_rustls::client::TlsStream as Tokio02RustlsTlsStream;
#[cfg(feature = "tokio1-rustls-tls")]
use tokio1_rustls::client::TlsStream as Tokio1RustlsTlsStream;

#[cfg(any(
    feature = "tokio02-native-tls",
    feature = "tokio02-rustls-tls",
    feature = "tokio1-native-tls",
    feature = "tokio1-rustls-tls",
    feature = "async-std1-native-tls",
    feature = "async-std1-rustls-tls"
))]
use super::InnerTlsParameters;
use super::TlsParameters;
use crate::transport::smtp::{error, Error};

/// A network stream
pub struct AsyncNetworkStream {
    inner: InnerAsyncNetworkStream,
}

/// Represents the different types of underlying network streams
// usually only one TLS backend at a time is going to be enabled,
// so clippy::large_enum_variant doesn't make sense here
#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
enum InnerAsyncNetworkStream {
    /// Plain Tokio 0.2 TCP stream
    #[cfg(feature = "tokio02")]
    Tokio02Tcp(Tokio02TcpStream),
    /// Encrypted Tokio 0.2 TCP stream
    #[cfg(feature = "tokio02-native-tls")]
    Tokio02NativeTls(Tokio02TlsStream<Tokio02TcpStream>),
    /// Encrypted Tokio 0.2 TCP stream
    #[cfg(feature = "tokio02-rustls-tls")]
    Tokio02RustlsTls(Tokio02RustlsTlsStream<Tokio02TcpStream>),
    /// Plain Tokio 1.x TCP stream
    #[cfg(feature = "tokio1")]
    Tokio1Tcp(Tokio1TcpStream),
    /// Encrypted Tokio 1.x TCP stream
    #[cfg(feature = "tokio1-native-tls")]
    Tokio1NativeTls(Tokio1TlsStream<Tokio1TcpStream>),
    /// Encrypted Tokio 1.x TCP stream
    #[cfg(feature = "tokio1-rustls-tls")]
    Tokio1RustlsTls(Tokio1RustlsTlsStream<Tokio1TcpStream>),
    /// Plain Tokio 1.x TCP stream
    #[cfg(feature = "async-std1")]
    AsyncStd1Tcp(AsyncStd1TcpStream),
    /// Encrypted Tokio 1.x TCP stream
    #[cfg(feature = "async-std1-native-tls")]
    AsyncStd1NativeTls(AsyncStd1TlsStream<AsyncStd1TcpStream>),
    /// Encrypted Tokio 1.x TCP stream
    #[cfg(feature = "async-std1-rustls-tls")]
    AsyncStd1RustlsTls(AsyncStd1RustlsTlsStream<AsyncStd1TcpStream>),
    /// Can't be built
    None,
}

impl AsyncNetworkStream {
    fn new(inner: InnerAsyncNetworkStream) -> Self {
        if let InnerAsyncNetworkStream::None = inner {
            debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
        }

        AsyncNetworkStream { inner }
    }

    /// Returns peer's address
    pub fn peer_addr(&self) -> IoResult<SocketAddr> {
        match self.inner {
            #[cfg(feature = "tokio02")]
            InnerAsyncNetworkStream::Tokio02Tcp(ref s) => s.peer_addr(),
            #[cfg(feature = "tokio02-native-tls")]
            InnerAsyncNetworkStream::Tokio02NativeTls(ref s) => {
                s.get_ref().get_ref().get_ref().peer_addr()
            }
            #[cfg(feature = "tokio02-rustls-tls")]
            InnerAsyncNetworkStream::Tokio02RustlsTls(ref s) => s.get_ref().0.peer_addr(),
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(ref s) => s.peer_addr(),
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(ref s) => {
                s.get_ref().get_ref().get_ref().peer_addr()
            }
            #[cfg(feature = "tokio1-rustls-tls")]
            InnerAsyncNetworkStream::Tokio1RustlsTls(ref s) => s.get_ref().0.peer_addr(),
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(ref s) => s.peer_addr(),
            #[cfg(feature = "async-std1-native-tls")]
            InnerAsyncNetworkStream::AsyncStd1NativeTls(ref s) => s.get_ref().peer_addr(),
            #[cfg(feature = "async-std1-rustls-tls")]
            InnerAsyncNetworkStream::AsyncStd1RustlsTls(ref s) => s.get_ref().0.peer_addr(),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
                Err(IoError::new(
                    ErrorKind::Other,
                    "InnerAsyncNetworkStream::None must never be built",
                ))
            }
        }
    }

    #[cfg(feature = "tokio02")]
    pub async fn connect_tokio02(
        hostname: &str,
        port: u16,
        tls_parameters: Option<TlsParameters>,
    ) -> Result<AsyncNetworkStream, Error> {
        let tcp_stream = Tokio02TcpStream::connect((hostname, port))
            .await
            .map_err(error::connection)?;

        let mut stream = AsyncNetworkStream::new(InnerAsyncNetworkStream::Tokio02Tcp(tcp_stream));
        if let Some(tls_parameters) = tls_parameters {
            stream.upgrade_tls(tls_parameters).await?;
        }
        Ok(stream)
    }

    #[cfg(feature = "tokio1")]
    pub async fn connect_tokio1(
        hostname: &str,
        port: u16,
        tls_parameters: Option<TlsParameters>,
    ) -> Result<AsyncNetworkStream, Error> {
        let tcp_stream = Tokio1TcpStream::connect((hostname, port))
            .await
            .map_err(error::connection)?;

        let mut stream = AsyncNetworkStream::new(InnerAsyncNetworkStream::Tokio1Tcp(tcp_stream));
        if let Some(tls_parameters) = tls_parameters {
            stream.upgrade_tls(tls_parameters).await?;
        }
        Ok(stream)
    }

    #[cfg(feature = "async-std1")]
    pub async fn connect_asyncstd1(
        hostname: &str,
        port: u16,
        tls_parameters: Option<TlsParameters>,
    ) -> Result<AsyncNetworkStream, Error> {
        let tcp_stream = AsyncStd1TcpStream::connect((hostname, port))
            .await
            .map_err(error::connection)?;

        let mut stream = AsyncNetworkStream::new(InnerAsyncNetworkStream::AsyncStd1Tcp(tcp_stream));
        if let Some(tls_parameters) = tls_parameters {
            stream.upgrade_tls(tls_parameters).await?;
        }
        Ok(stream)
    }

    pub async fn upgrade_tls(&mut self, tls_parameters: TlsParameters) -> Result<(), Error> {
        match &self.inner {
            #[cfg(all(
                feature = "tokio02",
                not(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))
            ))]
            InnerAsyncNetworkStream::Tokio02Tcp(_) => {
                let _ = tls_parameters;
                panic!("Trying to upgrade an AsyncNetworkStream without having enabled either the tokio02-native-tls or the tokio02-rustls-tls feature");
            }

            #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
            InnerAsyncNetworkStream::Tokio02Tcp(_) => {
                // get owned TcpStream
                let tcp_stream = mem::replace(&mut self.inner, InnerAsyncNetworkStream::None);
                let tcp_stream = match tcp_stream {
                    InnerAsyncNetworkStream::Tokio02Tcp(tcp_stream) => tcp_stream,
                    _ => unreachable!(),
                };

                self.inner = Self::upgrade_tokio02_tls(tcp_stream, tls_parameters)
                    .await
                    .map_err(error::connection)?;
                Ok(())
            }
            #[cfg(all(
                feature = "tokio1",
                not(any(feature = "tokio1-native-tls", feature = "tokio1-rustls-tls"))
            ))]
            InnerAsyncNetworkStream::Tokio1Tcp(_) => {
                let _ = tls_parameters;
                panic!("Trying to upgrade an AsyncNetworkStream without having enabled either the tokio1-native-tls or the tokio1-rustls-tls feature");
            }

            #[cfg(any(feature = "tokio1-native-tls", feature = "tokio1-rustls-tls"))]
            InnerAsyncNetworkStream::Tokio1Tcp(_) => {
                // get owned TcpStream
                let tcp_stream = mem::replace(&mut self.inner, InnerAsyncNetworkStream::None);
                let tcp_stream = match tcp_stream {
                    InnerAsyncNetworkStream::Tokio1Tcp(tcp_stream) => tcp_stream,
                    _ => unreachable!(),
                };

                self.inner = Self::upgrade_tokio1_tls(tcp_stream, tls_parameters)
                    .await
                    .map_err(error::connection)?;
                Ok(())
            }
            #[cfg(all(
                feature = "async-std1",
                not(any(feature = "async-std1-native-tls", feature = "async-std1-rustls-tls"))
            ))]
            InnerAsyncNetworkStream::AsyncStd1Tcp(_) => {
                let _ = tls_parameters;
                panic!("Trying to upgrade an AsyncNetworkStream without having enabled either the async-std1-native-tls or the async-std1-rustls-tls feature");
            }

            #[cfg(any(feature = "async-std1-native-tls", feature = "async-std1-rustls-tls"))]
            InnerAsyncNetworkStream::AsyncStd1Tcp(_) => {
                // get owned TcpStream
                let tcp_stream = mem::replace(&mut self.inner, InnerAsyncNetworkStream::None);
                let tcp_stream = match tcp_stream {
                    InnerAsyncNetworkStream::AsyncStd1Tcp(tcp_stream) => tcp_stream,
                    _ => unreachable!(),
                };

                self.inner = Self::upgrade_asyncstd1_tls(tcp_stream, tls_parameters)
                    .await
                    .map_err(error::connection)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    #[allow(unused_variables)]
    #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
    async fn upgrade_tokio02_tls(
        tcp_stream: Tokio02TcpStream,
        mut tls_parameters: TlsParameters,
    ) -> Result<InnerAsyncNetworkStream, Error> {
        let domain = mem::take(&mut tls_parameters.domain);

        match tls_parameters.connector {
            #[cfg(feature = "native-tls")]
            InnerTlsParameters::NativeTls(connector) => {
                #[cfg(not(feature = "tokio02-native-tls"))]
                panic!("built without the tokio02-native-tls feature");

                #[cfg(feature = "tokio02-native-tls")]
                return {
                    use tokio02_native_tls_crate::TlsConnector;

                    let connector = TlsConnector::from(connector);
                    let stream = connector
                        .connect(&domain, tcp_stream)
                        .await
                        .map_err(error::connection)?;
                    Ok(InnerAsyncNetworkStream::Tokio02NativeTls(stream))
                };
            }
            #[cfg(feature = "rustls-tls")]
            InnerTlsParameters::RustlsTls(config) => {
                #[cfg(not(feature = "tokio02-rustls-tls"))]
                panic!("built without the tokio02-rustls-tls feature");

                #[cfg(feature = "tokio02-rustls-tls")]
                return {
                    use tokio02_rustls::{webpki::DNSNameRef, TlsConnector};

                    let domain =
                        DNSNameRef::try_from_ascii_str(&domain).map_err(error::connection)?;

                    let connector = TlsConnector::from(Arc::new(config));
                    let stream = connector
                        .connect(domain, tcp_stream)
                        .await
                        .map_err(error::connection)?;
                    Ok(InnerAsyncNetworkStream::Tokio02RustlsTls(stream))
                };
            }
        }
    }

    #[allow(unused_variables)]
    #[cfg(any(feature = "tokio1-native-tls", feature = "tokio1-rustls-tls"))]
    async fn upgrade_tokio1_tls(
        tcp_stream: Tokio1TcpStream,
        mut tls_parameters: TlsParameters,
    ) -> Result<InnerAsyncNetworkStream, Error> {
        let domain = mem::take(&mut tls_parameters.domain);

        match tls_parameters.connector {
            #[cfg(feature = "native-tls")]
            InnerTlsParameters::NativeTls(connector) => {
                #[cfg(not(feature = "tokio1-native-tls"))]
                panic!("built without the tokio1-native-tls feature");

                #[cfg(feature = "tokio1-native-tls")]
                return {
                    use tokio1_native_tls_crate::TlsConnector;

                    let connector = TlsConnector::from(connector);
                    let stream = connector
                        .connect(&domain, tcp_stream)
                        .await
                        .map_err(error::connection)?;
                    Ok(InnerAsyncNetworkStream::Tokio1NativeTls(stream))
                };
            }
            #[cfg(feature = "rustls-tls")]
            InnerTlsParameters::RustlsTls(config) => {
                #[cfg(not(feature = "tokio1-rustls-tls"))]
                panic!("built without the tokio1-rustls-tls feature");

                #[cfg(feature = "tokio1-rustls-tls")]
                return {
                    use tokio1_rustls::{webpki::DNSNameRef, TlsConnector};

                    let domain =
                        DNSNameRef::try_from_ascii_str(&domain).map_err(error::connection)?;

                    let connector = TlsConnector::from(Arc::new(config));
                    let stream = connector
                        .connect(domain, tcp_stream)
                        .await
                        .map_err(error::connection)?;
                    Ok(InnerAsyncNetworkStream::Tokio1RustlsTls(stream))
                };
            }
        }
    }

    #[allow(unused_variables)]
    #[cfg(any(feature = "async-std1-native-tls", feature = "async-std1-rustls-tls"))]
    async fn upgrade_asyncstd1_tls(
        tcp_stream: AsyncStd1TcpStream,
        mut tls_parameters: TlsParameters,
    ) -> Result<InnerAsyncNetworkStream, Error> {
        let domain = mem::take(&mut tls_parameters.domain);

        match tls_parameters.connector {
            #[cfg(feature = "native-tls")]
            InnerTlsParameters::NativeTls(connector) => {
                panic!("native-tls isn't supported with async-std yet. See https://github.com/lettre/lettre/pull/531#issuecomment-757893531");

                /*
                #[cfg(not(feature = "async-std1-native-tls"))]
                panic!("built without the async-std1-native-tls feature");

                #[cfg(feature = "async-std1-native-tls")]
                return {
                    use async_native_tls::TlsConnector;

                    // TODO: fix
                    let connector: TlsConnector = todo!();
                    // let connector = TlsConnector::from(connector);
                    let stream = connector.connect(&domain, tcp_stream).await?;
                    Ok(InnerAsyncNetworkStream::AsyncStd1NativeTls(stream))
                };
                */
            }
            #[cfg(feature = "rustls-tls")]
            InnerTlsParameters::RustlsTls(config) => {
                #[cfg(not(feature = "async-std1-rustls-tls"))]
                panic!("built without the async-std1-rustls-tls feature");

                #[cfg(feature = "async-std1-rustls-tls")]
                return {
                    use async_rustls::{webpki::DNSNameRef, TlsConnector};

                    let domain =
                        DNSNameRef::try_from_ascii_str(&domain).map_err(error::connection)?;

                    let connector = TlsConnector::from(Arc::new(config));
                    let stream = connector
                        .connect(domain, tcp_stream)
                        .await
                        .map_err(error::connection)?;
                    Ok(InnerAsyncNetworkStream::AsyncStd1RustlsTls(stream))
                };
            }
        }
    }

    pub fn is_encrypted(&self) -> bool {
        match self.inner {
            #[cfg(feature = "tokio02")]
            InnerAsyncNetworkStream::Tokio02Tcp(_) => false,
            #[cfg(feature = "tokio02-native-tls")]
            InnerAsyncNetworkStream::Tokio02NativeTls(_) => true,
            #[cfg(feature = "tokio02-rustls-tls")]
            InnerAsyncNetworkStream::Tokio02RustlsTls(_) => true,
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(_) => false,
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(_) => true,
            #[cfg(feature = "tokio1-rustls-tls")]
            InnerAsyncNetworkStream::Tokio1RustlsTls(_) => true,
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(_) => false,
            #[cfg(feature = "async-std1-native-tls")]
            InnerAsyncNetworkStream::AsyncStd1NativeTls(_) => true,
            #[cfg(feature = "async-std1-rustls-tls")]
            InnerAsyncNetworkStream::AsyncStd1RustlsTls(_) => true,
            InnerAsyncNetworkStream::None => false,
        }
    }
}

impl FuturesAsyncRead for AsyncNetworkStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<IoResult<usize>> {
        match self.inner {
            #[cfg(feature = "tokio02")]
            InnerAsyncNetworkStream::Tokio02Tcp(ref mut s) => Pin::new(s).poll_read(cx, buf),
            #[cfg(feature = "tokio02-native-tls")]
            InnerAsyncNetworkStream::Tokio02NativeTls(ref mut s) => Pin::new(s).poll_read(cx, buf),
            #[cfg(feature = "tokio02-rustls-tls")]
            InnerAsyncNetworkStream::Tokio02RustlsTls(ref mut s) => Pin::new(s).poll_read(cx, buf),
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(ref mut s) => {
                let mut b = Tokio1ReadBuf::new(buf);
                match Pin::new(s).poll_read(cx, &mut b) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(b.filled().len())),
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            }
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(ref mut s) => {
                let mut b = Tokio1ReadBuf::new(buf);
                match Pin::new(s).poll_read(cx, &mut b) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(b.filled().len())),
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            }
            #[cfg(feature = "tokio1-rustls-tls")]
            InnerAsyncNetworkStream::Tokio1RustlsTls(ref mut s) => {
                let mut b = Tokio1ReadBuf::new(buf);
                match Pin::new(s).poll_read(cx, &mut b) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(b.filled().len())),
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            }
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(ref mut s) => Pin::new(s).poll_read(cx, buf),
            #[cfg(feature = "async-std1-native-tls")]
            InnerAsyncNetworkStream::AsyncStd1NativeTls(ref mut s) => {
                Pin::new(s).poll_read(cx, buf)
            }
            #[cfg(feature = "async-std1-rustls-tls")]
            InnerAsyncNetworkStream::AsyncStd1RustlsTls(ref mut s) => {
                Pin::new(s).poll_read(cx, buf)
            }
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
                Poll::Ready(Ok(0))
            }
        }
    }
}

impl FuturesAsyncWrite for AsyncNetworkStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        match self.inner {
            #[cfg(feature = "tokio02")]
            InnerAsyncNetworkStream::Tokio02Tcp(ref mut s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "tokio02-native-tls")]
            InnerAsyncNetworkStream::Tokio02NativeTls(ref mut s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "tokio02-rustls-tls")]
            InnerAsyncNetworkStream::Tokio02RustlsTls(ref mut s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(ref mut s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(ref mut s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "tokio1-rustls-tls")]
            InnerAsyncNetworkStream::Tokio1RustlsTls(ref mut s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(ref mut s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "async-std1-native-tls")]
            InnerAsyncNetworkStream::AsyncStd1NativeTls(ref mut s) => {
                Pin::new(s).poll_write(cx, buf)
            }
            #[cfg(feature = "async-std1-rustls-tls")]
            InnerAsyncNetworkStream::AsyncStd1RustlsTls(ref mut s) => {
                Pin::new(s).poll_write(cx, buf)
            }
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
                Poll::Ready(Ok(0))
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        match self.inner {
            #[cfg(feature = "tokio02")]
            InnerAsyncNetworkStream::Tokio02Tcp(ref mut s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "tokio02-native-tls")]
            InnerAsyncNetworkStream::Tokio02NativeTls(ref mut s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "tokio02-rustls-tls")]
            InnerAsyncNetworkStream::Tokio02RustlsTls(ref mut s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(ref mut s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(ref mut s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "tokio1-rustls-tls")]
            InnerAsyncNetworkStream::Tokio1RustlsTls(ref mut s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(ref mut s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "async-std1-native-tls")]
            InnerAsyncNetworkStream::AsyncStd1NativeTls(ref mut s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "async-std1-rustls-tls")]
            InnerAsyncNetworkStream::AsyncStd1RustlsTls(ref mut s) => Pin::new(s).poll_flush(cx),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
                Poll::Ready(Ok(()))
            }
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        match self.inner {
            #[cfg(feature = "tokio02")]
            InnerAsyncNetworkStream::Tokio02Tcp(ref mut s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(feature = "tokio02-native-tls")]
            InnerAsyncNetworkStream::Tokio02NativeTls(ref mut s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(feature = "tokio02-rustls-tls")]
            InnerAsyncNetworkStream::Tokio02RustlsTls(ref mut s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(ref mut s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(ref mut s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(feature = "tokio1-rustls-tls")]
            InnerAsyncNetworkStream::Tokio1RustlsTls(ref mut s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(ref mut s) => Pin::new(s).poll_close(cx),
            #[cfg(feature = "async-std1-native-tls")]
            InnerAsyncNetworkStream::AsyncStd1NativeTls(ref mut s) => Pin::new(s).poll_close(cx),
            #[cfg(feature = "async-std1-rustls-tls")]
            InnerAsyncNetworkStream::AsyncStd1RustlsTls(ref mut s) => Pin::new(s).poll_close(cx),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
                Poll::Ready(Ok(()))
            }
        }
    }
}
