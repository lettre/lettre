use std::{
    fmt, io, mem,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

#[cfg(feature = "async-std1")]
use async_std::net::{TcpStream as AsyncStd1TcpStream, ToSocketAddrs as AsyncStd1ToSocketAddrs};
use futures_io::{
    AsyncRead as FuturesAsyncRead, AsyncWrite as FuturesAsyncWrite, Error as IoError,
    Result as IoResult,
};
#[cfg(feature = "async-std1-rustls")]
use futures_rustls::client::TlsStream as AsyncStd1RustlsStream;
#[cfg(any(feature = "tokio1-rustls", feature = "async-std1-rustls"))]
use rustls::pki_types::ServerName;
#[cfg(feature = "tokio1-boring-tls")]
use tokio1_boring::SslStream as Tokio1SslStream;
#[cfg(feature = "tokio1")]
use tokio1_crate::io::{AsyncRead, AsyncWrite, ReadBuf as Tokio1ReadBuf};
#[cfg(feature = "tokio1")]
use tokio1_crate::net::{
    TcpSocket as Tokio1TcpSocket, TcpStream as Tokio1TcpStream,
    ToSocketAddrs as Tokio1ToSocketAddrs,
};
#[cfg(feature = "tokio1-native-tls")]
use tokio1_native_tls_crate::TlsStream as Tokio1TlsStream;
#[cfg(feature = "tokio1-rustls")]
use tokio1_rustls::client::TlsStream as Tokio1RustlsStream;

#[cfg(any(
    feature = "tokio1-native-tls",
    feature = "tokio1-rustls",
    feature = "tokio1-boring-tls",
    feature = "async-std1-rustls"
))]
use super::InnerTlsParameters;
use super::TlsParameters;
#[cfg(feature = "tokio1")]
use crate::transport::smtp::client::net::resolved_address_filter;
use crate::transport::smtp::{error, Error};

/// A network stream
#[derive(Debug)]
#[deprecated(
    since = "0.11.14",
    note = "This struct was not meant to be made public"
)]
pub struct AsyncNetworkStream {
    inner: InnerAsyncNetworkStream,
}

#[cfg(feature = "tokio1")]
pub trait AsyncTokioStream: AsyncRead + AsyncWrite + Send + Sync + Unpin + fmt::Debug {
    fn peer_addr(&self) -> io::Result<SocketAddr>;
}

#[cfg(feature = "tokio1")]
impl AsyncTokioStream for Tokio1TcpStream {
    fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.peer_addr()
    }
}

/// Represents the different types of underlying network streams
// usually only one TLS backend at a time is going to be enabled,
// so clippy::large_enum_variant doesn't make sense here
#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
#[derive(Debug)]
enum InnerAsyncNetworkStream {
    /// Plain Tokio 1.x TCP stream
    #[cfg(feature = "tokio1")]
    Tokio1Tcp(Box<dyn AsyncTokioStream>),
    /// Encrypted Tokio 1.x TCP stream
    #[cfg(feature = "tokio1-native-tls")]
    Tokio1NativeTls(Tokio1TlsStream<Box<dyn AsyncTokioStream>>),
    /// Encrypted Tokio 1.x TCP stream
    #[cfg(feature = "tokio1-rustls")]
    Tokio1Rustls(Tokio1RustlsStream<Box<dyn AsyncTokioStream>>),
    /// Encrypted Tokio 1.x TCP stream
    #[cfg(feature = "tokio1-boring-tls")]
    Tokio1BoringTls(Tokio1SslStream<Box<dyn AsyncTokioStream>>),
    /// Plain Tokio 1.x TCP stream
    #[cfg(feature = "async-std1")]
    AsyncStd1Tcp(AsyncStd1TcpStream),
    /// Encrypted Tokio 1.x TCP stream
    #[cfg(feature = "async-std1-rustls")]
    AsyncStd1Rustls(AsyncStd1RustlsStream<AsyncStd1TcpStream>),
    /// Can't be built
    None,
}

#[allow(deprecated)]
impl AsyncNetworkStream {
    fn new(inner: InnerAsyncNetworkStream) -> Self {
        if let InnerAsyncNetworkStream::None = inner {
            debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
        }

        AsyncNetworkStream { inner }
    }

    /// Returns peer's address
    pub fn peer_addr(&self) -> IoResult<SocketAddr> {
        match &self.inner {
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(s) => s.peer_addr(),
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(s) => {
                s.get_ref().get_ref().get_ref().peer_addr()
            }
            #[cfg(feature = "tokio1-rustls")]
            InnerAsyncNetworkStream::Tokio1Rustls(s) => s.get_ref().0.peer_addr(),
            #[cfg(feature = "tokio1-boring-tls")]
            InnerAsyncNetworkStream::Tokio1BoringTls(s) => s.get_ref().peer_addr(),
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(s) => s.peer_addr(),
            #[cfg(feature = "async-std1-rustls")]
            InnerAsyncNetworkStream::AsyncStd1Rustls(s) => s.get_ref().0.peer_addr(),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
                Err(IoError::other(
                    "InnerAsyncNetworkStream::None must never be built",
                ))
            }
        }
    }

    #[cfg(feature = "tokio1")]
    pub fn use_existing_tokio1(stream: Box<dyn AsyncTokioStream>) -> AsyncNetworkStream {
        AsyncNetworkStream::new(InnerAsyncNetworkStream::Tokio1Tcp(stream))
    }

    #[cfg(feature = "tokio1")]
    pub async fn connect_tokio1<T: Tokio1ToSocketAddrs>(
        server: T,
        timeout: Option<Duration>,
        tls_parameters: Option<TlsParameters>,
        local_addr: Option<IpAddr>,
    ) -> Result<AsyncNetworkStream, Error> {
        async fn try_connect<T: Tokio1ToSocketAddrs>(
            server: T,
            timeout: Option<Duration>,
            local_addr: Option<IpAddr>,
        ) -> Result<Tokio1TcpStream, Error> {
            let addrs = tokio1_crate::net::lookup_host(server)
                .await
                .map_err(error::connection)?
                .filter(|resolved_addr| resolved_address_filter(resolved_addr, local_addr));

            let mut last_err = None;

            for addr in addrs {
                let socket = match addr.ip() {
                    IpAddr::V4(_) => Tokio1TcpSocket::new_v4(),
                    IpAddr::V6(_) => Tokio1TcpSocket::new_v6(),
                }
                .map_err(error::connection)?;
                if let Some(local_addr) = local_addr {
                    socket
                        .bind(SocketAddr::new(local_addr, 0))
                        .map_err(error::connection)?;
                }

                let connect_future = socket.connect(addr);
                if let Some(timeout) = timeout {
                    match tokio1_crate::time::timeout(timeout, connect_future).await {
                        Ok(Ok(stream)) => return Ok(stream),
                        Ok(Err(err)) => last_err = Some(err),
                        Err(_) => {
                            last_err = Some(io::Error::new(
                                io::ErrorKind::TimedOut,
                                "connection timed out",
                            ));
                        }
                    }
                } else {
                    match connect_future.await {
                        Ok(stream) => return Ok(stream),
                        Err(err) => last_err = Some(err),
                    }
                }
            }

            Err(match last_err {
                Some(last_err) => error::connection(last_err),
                None => error::connection("could not resolve to any supported address"),
            })
        }

        let tcp_stream = try_connect(server, timeout, local_addr).await?;
        let mut stream =
            AsyncNetworkStream::new(InnerAsyncNetworkStream::Tokio1Tcp(Box::new(tcp_stream)));
        if let Some(tls_parameters) = tls_parameters {
            stream.upgrade_tls(tls_parameters).await?;
        }
        Ok(stream)
    }

    #[cfg(feature = "async-std1")]
    pub async fn connect_asyncstd1<T: AsyncStd1ToSocketAddrs>(
        server: T,
        timeout: Option<Duration>,
        tls_parameters: Option<TlsParameters>,
    ) -> Result<AsyncNetworkStream, Error> {
        // Unfortunately, there doesn't currently seem to be a way to set the local address.
        // Whilst we can create a AsyncStd1TcpStream from an existing socket, it needs to first have
        // been connected, which is a blocking operation.
        async fn try_connect_timeout<T: AsyncStd1ToSocketAddrs>(
            server: T,
            timeout: Duration,
        ) -> Result<AsyncStd1TcpStream, Error> {
            let addrs = server.to_socket_addrs().await.map_err(error::connection)?;

            let mut last_err = None;

            for addr in addrs {
                let connect_future = AsyncStd1TcpStream::connect(&addr);
                match async_std::future::timeout(timeout, connect_future).await {
                    Ok(Ok(stream)) => return Ok(stream),
                    Ok(Err(err)) => last_err = Some(err),
                    Err(_) => {
                        last_err = Some(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "connection timed out",
                        ));
                    }
                }
            }

            Err(match last_err {
                Some(last_err) => error::connection(last_err),
                None => error::connection("could not resolve to any address"),
            })
        }

        let tcp_stream = match timeout {
            Some(t) => try_connect_timeout(server, t).await?,
            None => AsyncStd1TcpStream::connect(server)
                .await
                .map_err(error::connection)?,
        };

        let mut stream = AsyncNetworkStream::new(InnerAsyncNetworkStream::AsyncStd1Tcp(tcp_stream));
        if let Some(tls_parameters) = tls_parameters {
            stream.upgrade_tls(tls_parameters).await?;
        }
        Ok(stream)
    }

    pub async fn upgrade_tls(&mut self, tls_parameters: TlsParameters) -> Result<(), Error> {
        match &self.inner {
            #[cfg(all(
                feature = "tokio1",
                not(any(
                    feature = "tokio1-native-tls",
                    feature = "tokio1-rustls",
                    feature = "tokio1-boring-tls"
                ))
            ))]
            InnerAsyncNetworkStream::Tokio1Tcp(_) => {
                let _ = tls_parameters;
                panic!("Trying to upgrade an AsyncNetworkStream without having enabled either the tokio1-native-tls or the tokio1-rustls feature");
            }

            #[cfg(any(
                feature = "tokio1-native-tls",
                feature = "tokio1-rustls",
                feature = "tokio1-boring-tls"
            ))]
            InnerAsyncNetworkStream::Tokio1Tcp(_) => {
                // get owned TcpStream
                let tcp_stream = mem::replace(&mut self.inner, InnerAsyncNetworkStream::None);
                let InnerAsyncNetworkStream::Tokio1Tcp(tcp_stream) = tcp_stream else {
                    unreachable!()
                };

                self.inner = Self::upgrade_tokio1_tls(tcp_stream, tls_parameters)
                    .await
                    .map_err(error::connection)?;
                Ok(())
            }
            #[cfg(all(feature = "async-std1", not(feature = "async-std1-rustls")))]
            InnerAsyncNetworkStream::AsyncStd1Tcp(_) => {
                let _ = tls_parameters;
                panic!("Trying to upgrade an AsyncNetworkStream without having enabled the async-std1-rustls feature");
            }

            #[cfg(feature = "async-std1-rustls")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(_) => {
                // get owned TcpStream
                let tcp_stream = mem::replace(&mut self.inner, InnerAsyncNetworkStream::None);
                let InnerAsyncNetworkStream::AsyncStd1Tcp(tcp_stream) = tcp_stream else {
                    unreachable!()
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
    #[cfg(any(
        feature = "tokio1-native-tls",
        feature = "tokio1-rustls",
        feature = "tokio1-boring-tls"
    ))]
    async fn upgrade_tokio1_tls(
        tcp_stream: Box<dyn AsyncTokioStream>,
        tls_parameters: TlsParameters,
    ) -> Result<InnerAsyncNetworkStream, Error> {
        let domain = tls_parameters.domain().to_owned();

        match tls_parameters.connector {
            #[cfg(feature = "native-tls")]
            InnerTlsParameters::NativeTls { connector } => {
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
            #[cfg(feature = "rustls")]
            InnerTlsParameters::Rustls { config } => {
                #[cfg(not(feature = "tokio1-rustls"))]
                panic!("built without the tokio1-rustls feature");

                #[cfg(feature = "tokio1-rustls")]
                return {
                    use tokio1_rustls::TlsConnector;

                    let domain = ServerName::try_from(domain.as_str())
                        .map_err(|_| error::connection("domain isn't a valid DNS name"))?;

                    let connector = TlsConnector::from(config);
                    let stream = connector
                        .connect(domain.to_owned(), tcp_stream)
                        .await
                        .map_err(error::connection)?;
                    Ok(InnerAsyncNetworkStream::Tokio1Rustls(stream))
                };
            }
            #[cfg(feature = "boring-tls")]
            InnerTlsParameters::BoringTls {
                connector,
                accept_invalid_hostnames,
            } => {
                #[cfg(not(feature = "tokio1-boring-tls"))]
                panic!("built without the tokio1-boring-tls feature");

                #[cfg(feature = "tokio1-boring-tls")]
                return {
                    let mut config = connector.configure().map_err(error::connection)?;
                    config.set_verify_hostname(accept_invalid_hostnames);

                    let stream = tokio1_boring::connect(config, &domain, tcp_stream)
                        .await
                        .map_err(error::connection)?;
                    Ok(InnerAsyncNetworkStream::Tokio1BoringTls(stream))
                };
            }
        }
    }

    #[allow(unused_variables)]
    #[cfg(feature = "async-std1-rustls")]
    async fn upgrade_asyncstd1_tls(
        tcp_stream: AsyncStd1TcpStream,
        mut tls_parameters: TlsParameters,
    ) -> Result<InnerAsyncNetworkStream, Error> {
        let domain = mem::take(&mut tls_parameters.domain);

        match tls_parameters.connector {
            #[cfg(feature = "native-tls")]
            InnerTlsParameters::NativeTls { connector } => {
                panic!("native-tls isn't supported with async-std yet. See https://github.com/lettre/lettre/pull/531#issuecomment-757893531");
            }
            #[cfg(feature = "rustls")]
            InnerTlsParameters::Rustls { config } => {
                #[cfg(not(feature = "async-std1-rustls"))]
                panic!("built without the async-std1-rustls feature");

                #[cfg(feature = "async-std1-rustls")]
                return {
                    use futures_rustls::TlsConnector;

                    let domain = ServerName::try_from(domain.as_str())
                        .map_err(|_| error::connection("domain isn't a valid DNS name"))?;

                    let connector = TlsConnector::from(config);
                    let stream = connector
                        .connect(domain.to_owned(), tcp_stream)
                        .await
                        .map_err(error::connection)?;
                    Ok(InnerAsyncNetworkStream::AsyncStd1Rustls(stream))
                };
            }
            #[cfg(feature = "boring-tls")]
            InnerTlsParameters::BoringTls { .. } => {
                panic!("boring-tls isn't supported with async-std yet.");
            }
        }
    }

    pub fn is_encrypted(&self) -> bool {
        match &self.inner {
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(_) => false,
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(_) => true,
            #[cfg(feature = "tokio1-rustls")]
            InnerAsyncNetworkStream::Tokio1Rustls(_) => true,
            #[cfg(feature = "tokio1-boring-tls")]
            InnerAsyncNetworkStream::Tokio1BoringTls(_) => true,
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(_) => false,
            #[cfg(feature = "async-std1-rustls")]
            InnerAsyncNetworkStream::AsyncStd1Rustls(_) => true,
            InnerAsyncNetworkStream::None => false,
        }
    }

    #[cfg(feature = "boring-tls")]
    pub fn tls_verify_result(&self) -> Result<(), Error> {
        match &self.inner {
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(_) => {
                Err(error::client("Connection is not encrypted"))
            }
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(_) => panic!("Unsupported"),
            #[cfg(feature = "tokio1-rustls")]
            InnerAsyncNetworkStream::Tokio1Rustls(_) => panic!("Unsupported"),
            #[cfg(feature = "tokio1-boring-tls")]
            InnerAsyncNetworkStream::Tokio1BoringTls(stream) => {
                stream.ssl().verify_result().map_err(error::tls)
            }
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(_) => {
                Err(error::client("Connection is not encrypted"))
            }
            #[cfg(feature = "async-std1-rustls")]
            InnerAsyncNetworkStream::AsyncStd1Rustls(_) => panic!("Unsupported"),
            InnerAsyncNetworkStream::None => panic!("InnerNetworkStream::None must never be built"),
        }
    }
    pub fn certificate_chain(&self) -> Result<Vec<Vec<u8>>, Error> {
        match &self.inner {
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(_) => {
                Err(error::client("Connection is not encrypted"))
            }
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(_) => panic!("Unsupported"),
            #[cfg(feature = "tokio1-rustls")]
            InnerAsyncNetworkStream::Tokio1Rustls(stream) => Ok(stream
                .get_ref()
                .1
                .peer_certificates()
                .unwrap()
                .iter()
                .map(|c| c.to_vec())
                .collect()),
            #[cfg(feature = "tokio1-boring-tls")]
            InnerAsyncNetworkStream::Tokio1BoringTls(stream) => Ok(stream
                .ssl()
                .peer_cert_chain()
                .unwrap()
                .iter()
                .map(|c| c.to_der().map_err(error::tls))
                .collect::<Result<Vec<_>, _>>()?),
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(_) => {
                Err(error::client("Connection is not encrypted"))
            }
            #[cfg(feature = "async-std1-rustls")]
            InnerAsyncNetworkStream::AsyncStd1Rustls(stream) => Ok(stream
                .get_ref()
                .1
                .peer_certificates()
                .unwrap()
                .iter()
                .map(|c| c.to_vec())
                .collect()),
            InnerAsyncNetworkStream::None => panic!("InnerNetworkStream::None must never be built"),
        }
    }

    pub fn peer_certificate(&self) -> Result<Vec<u8>, Error> {
        match &self.inner {
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(_) => {
                Err(error::client("Connection is not encrypted"))
            }
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(stream) => Ok(stream
                .get_ref()
                .peer_certificate()
                .map_err(error::tls)?
                .unwrap()
                .to_der()
                .map_err(error::tls)?),
            #[cfg(feature = "tokio1-rustls")]
            InnerAsyncNetworkStream::Tokio1Rustls(stream) => Ok(stream
                .get_ref()
                .1
                .peer_certificates()
                .unwrap()
                .first()
                .unwrap()
                .to_vec()),
            #[cfg(feature = "tokio1-boring-tls")]
            InnerAsyncNetworkStream::Tokio1BoringTls(stream) => Ok(stream
                .ssl()
                .peer_certificate()
                .unwrap()
                .to_der()
                .map_err(error::tls)?),
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(_) => {
                Err(error::client("Connection is not encrypted"))
            }
            #[cfg(feature = "async-std1-rustls")]
            InnerAsyncNetworkStream::AsyncStd1Rustls(stream) => Ok(stream
                .get_ref()
                .1
                .peer_certificates()
                .unwrap()
                .first()
                .unwrap()
                .to_vec()),
            InnerAsyncNetworkStream::None => panic!("InnerNetworkStream::None must never be built"),
        }
    }
}

#[allow(deprecated)]
impl FuturesAsyncRead for AsyncNetworkStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<IoResult<usize>> {
        match &mut self.inner {
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(s) => {
                let mut b = Tokio1ReadBuf::new(buf);
                match Pin::new(s).poll_read(cx, &mut b) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(b.filled().len())),
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            }
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(s) => {
                let mut b = Tokio1ReadBuf::new(buf);
                match Pin::new(s).poll_read(cx, &mut b) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(b.filled().len())),
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            }
            #[cfg(feature = "tokio1-rustls")]
            InnerAsyncNetworkStream::Tokio1Rustls(s) => {
                let mut b = Tokio1ReadBuf::new(buf);
                match Pin::new(s).poll_read(cx, &mut b) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(b.filled().len())),
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            }
            #[cfg(feature = "tokio1-boring-tls")]
            InnerAsyncNetworkStream::Tokio1BoringTls(s) => {
                let mut b = Tokio1ReadBuf::new(buf);
                match Pin::new(s).poll_read(cx, &mut b) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(b.filled().len())),
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            }
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(s) => Pin::new(s).poll_read(cx, buf),
            #[cfg(feature = "async-std1-rustls")]
            InnerAsyncNetworkStream::AsyncStd1Rustls(s) => Pin::new(s).poll_read(cx, buf),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
                Poll::Ready(Ok(0))
            }
        }
    }
}

#[allow(deprecated)]
impl FuturesAsyncWrite for AsyncNetworkStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        match &mut self.inner {
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "tokio1-rustls")]
            InnerAsyncNetworkStream::Tokio1Rustls(s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "tokio1-boring-tls")]
            InnerAsyncNetworkStream::Tokio1BoringTls(s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(feature = "async-std1-rustls")]
            InnerAsyncNetworkStream::AsyncStd1Rustls(s) => Pin::new(s).poll_write(cx, buf),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
                Poll::Ready(Ok(0))
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        match &mut self.inner {
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "tokio1-rustls")]
            InnerAsyncNetworkStream::Tokio1Rustls(s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "tokio1-boring-tls")]
            InnerAsyncNetworkStream::Tokio1BoringTls(s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "async-std1-rustls")]
            InnerAsyncNetworkStream::AsyncStd1Rustls(s) => Pin::new(s).poll_flush(cx),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
                Poll::Ready(Ok(()))
            }
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        match &mut self.inner {
            #[cfg(feature = "tokio1")]
            InnerAsyncNetworkStream::Tokio1Tcp(s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(feature = "tokio1-native-tls")]
            InnerAsyncNetworkStream::Tokio1NativeTls(s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(feature = "tokio1-rustls")]
            InnerAsyncNetworkStream::Tokio1Rustls(s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(feature = "tokio1-boring-tls")]
            InnerAsyncNetworkStream::Tokio1BoringTls(s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(feature = "async-std1")]
            InnerAsyncNetworkStream::AsyncStd1Tcp(s) => Pin::new(s).poll_close(cx),
            #[cfg(feature = "async-std1-rustls")]
            InnerAsyncNetworkStream::AsyncStd1Rustls(s) => Pin::new(s).poll_close(cx),
            InnerAsyncNetworkStream::None => {
                debug_assert!(false, "InnerAsyncNetworkStream::None must never be built");
                Poll::Ready(Ok(()))
            }
        }
    }
}
