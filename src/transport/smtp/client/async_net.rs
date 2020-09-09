#[cfg(feature = "tokio02-rustls-tls")]
use std::sync::Arc;
use std::{
    net::{Shutdown, SocketAddr},
    pin::Pin,
    task::{Context, Poll},
};

use futures_io::{Error as IoError, ErrorKind, Result as IoResult};
#[cfg(feature = "tokio02")]
use tokio02_crate::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "tokio02")]
use tokio02_crate::net::TcpStream;

#[cfg(feature = "tokio02-native-tls")]
use tokio02_native_tls_crate::TlsStream;

#[cfg(feature = "tokio02-rustls-tls")]
use tokio02_rustls::client::TlsStream as RustlsTlsStream;

#[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
use super::InnerTlsParameters;
use super::TlsParameters;
use crate::transport::smtp::Error;

/// A network stream
pub struct AsyncNetworkStream {
    inner: InnerAsyncNetworkStream,
}

/// Represents the different types of underlying network streams
#[allow(dead_code)]
enum InnerAsyncNetworkStream {
    /// Plain TCP stream
    #[cfg(feature = "tokio02")]
    Tokio02Tcp(TcpStream),
    /// Encrypted TCP stream
    #[cfg(feature = "tokio02-native-tls")]
    Tokio02NativeTls(TlsStream<TcpStream>),
    /// Encrypted TCP stream
    #[cfg(feature = "tokio02-rustls-tls")]
    Tokio02RustlsTls(Box<RustlsTlsStream<TcpStream>>),
    /// Can't be built
    None,
}

impl AsyncNetworkStream {
    fn new(inner: InnerAsyncNetworkStream) -> Self {
        if let InnerAsyncNetworkStream::None = inner {
            debug_assert!(false, "InnerAsyncNetworkStream::None should never be built");
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
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None should never be built");
                Err(IoError::new(
                    ErrorKind::Other,
                    "InnerAsyncNetworkStream::None should never be built",
                ))
            }
        }
    }

    /// Shutdowns the connection
    pub fn shutdown(&self, how: Shutdown) -> IoResult<()> {
        match self.inner {
            #[cfg(feature = "tokio02")]
            InnerAsyncNetworkStream::Tokio02Tcp(ref s) => s.shutdown(how),
            #[cfg(feature = "tokio02-native-tls")]
            InnerAsyncNetworkStream::Tokio02NativeTls(ref s) => {
                s.get_ref().get_ref().get_ref().shutdown(how)
            }
            #[cfg(feature = "tokio02-rustls-tls")]
            InnerAsyncNetworkStream::Tokio02RustlsTls(ref s) => s.get_ref().0.shutdown(how),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None should never be built");
                Ok(())
            }
        }
    }

    #[cfg(feature = "tokio02")]
    pub async fn connect_tokio02(
        hostname: &str,
        port: u16,
        tls_parameters: Option<TlsParameters>,
    ) -> Result<AsyncNetworkStream, Error> {
        let tcp_stream = TcpStream::connect((hostname, port)).await?;

        let mut stream = AsyncNetworkStream::new(InnerAsyncNetworkStream::Tokio02Tcp(tcp_stream));
        if let Some(tls_parameters) = tls_parameters {
            stream.upgrade_tls(tls_parameters).await?;
        }
        Ok(stream)
    }

    pub async fn upgrade_tls(&mut self, tls_parameters: TlsParameters) -> Result<(), Error> {
        match &self.inner {
            #[cfg(not(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls")))]
            InnerAsyncNetworkStream::Tokio02Tcp(_) => {
                let _ = tls_parameters;
                panic!("Trying to upgrade an AsyncNetworkStream without having enabled either the tokio02-native-tls or the tokio02-rustls-tls feature");
            }

            #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
            InnerAsyncNetworkStream::Tokio02Tcp(_) => {
                // get owned TcpStream
                let tcp_stream = std::mem::replace(&mut self.inner, InnerAsyncNetworkStream::None);
                let tcp_stream = match tcp_stream {
                    InnerAsyncNetworkStream::Tokio02Tcp(tcp_stream) => tcp_stream,
                    _ => unreachable!(),
                };

                self.inner = Self::upgrade_tokio02_tls(tcp_stream, tls_parameters).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    #[allow(unused_variables)]
    #[cfg(any(feature = "tokio02-native-tls", feature = "tokio02-rustls-tls"))]
    async fn upgrade_tokio02_tls(
        tcp_stream: TcpStream,
        mut tls_parameters: TlsParameters,
    ) -> Result<InnerAsyncNetworkStream, Error> {
        let domain = std::mem::take(&mut tls_parameters.domain);

        match tls_parameters.connector {
            #[cfg(feature = "native-tls")]
            InnerTlsParameters::NativeTls(connector) => {
                #[cfg(not(feature = "tokio02-native-tls"))]
                panic!("built without the tokio02-native-tls feature");

                #[cfg(feature = "tokio02-native-tls")]
                return {
                    use tokio02_native_tls_crate::TlsConnector;

                    let connector = TlsConnector::from(connector);
                    let stream = connector.connect(&domain, tcp_stream).await?;
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

                    let domain = DNSNameRef::try_from_ascii_str(&domain)?;

                    let connector = TlsConnector::from(Arc::new(config));
                    let stream = connector.connect(domain, tcp_stream).await?;
                    Ok(InnerAsyncNetworkStream::Tokio02RustlsTls(Box::new(stream)))
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
            InnerAsyncNetworkStream::None => false,
        }
    }
}

impl futures_io::AsyncRead for AsyncNetworkStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<IoResult<usize>> {
        match self.inner {
            #[cfg(feature = "tokio02")]
            InnerAsyncNetworkStream::Tokio02Tcp(ref mut s) => Pin::new(s).poll_read(cx, buf),
            #[cfg(feature = "tokio02-native-tls")]
            InnerAsyncNetworkStream::Tokio02NativeTls(ref mut s) => Pin::new(s).poll_read(cx, buf),
            #[cfg(feature = "tokio02-rustls-tls")]
            InnerAsyncNetworkStream::Tokio02RustlsTls(ref mut s) => Pin::new(s).poll_read(cx, buf),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None should never be built");
                Poll::Ready(Ok(0))
            }
        }
    }
}

impl futures_io::AsyncWrite for AsyncNetworkStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<IoResult<usize>> {
        match self.inner {
            #[cfg(feature = "tokio02")]
            InnerAsyncNetworkStream::Tokio02Tcp(ref mut s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "tokio02-native-tls")]
            InnerAsyncNetworkStream::Tokio02NativeTls(ref mut s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "tokio02-rustls-tls")]
            InnerAsyncNetworkStream::Tokio02RustlsTls(ref mut s) => Pin::new(s).poll_write(cx, buf),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None should never be built");
                Poll::Ready(Ok(0))
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<IoResult<()>> {
        match self.inner {
            #[cfg(feature = "tokio02")]
            InnerAsyncNetworkStream::Tokio02Tcp(ref mut s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "tokio02-native-tls")]
            InnerAsyncNetworkStream::Tokio02NativeTls(ref mut s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "tokio02-rustls-tls")]
            InnerAsyncNetworkStream::Tokio02RustlsTls(ref mut s) => Pin::new(s).poll_flush(cx),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None should never be built");
                Poll::Ready(Ok(()))
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<IoResult<()>> {
        Poll::Ready(self.shutdown(Shutdown::Write))
    }
}
