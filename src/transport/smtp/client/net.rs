#[cfg(feature = "rustls-tls")]
use std::sync::Arc;
use std::{
    io::{self, Read, Write},
    mem,
    net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpStream, ToSocketAddrs},
    time::Duration,
};

#[cfg(feature = "native-tls")]
use native_tls::TlsStream;

#[cfg(feature = "rustls-tls")]
use rustls::{ClientSession, StreamOwned};

#[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
use super::InnerTlsParameters;
use super::TlsParameters;
use crate::transport::smtp::{error, Error};

/// A network stream
pub struct NetworkStream {
    inner: InnerNetworkStream,
}

/// Represents the different types of underlying network streams
// usually only one TLS backend at a time is going to be enabled,
// so clippy::large_enum_variant doesn't make sense here
#[allow(clippy::large_enum_variant)]
enum InnerNetworkStream {
    /// Plain TCP stream
    Tcp(TcpStream),
    /// Encrypted TCP stream
    #[cfg(feature = "native-tls")]
    NativeTls(TlsStream<TcpStream>),
    /// Encrypted TCP stream
    #[cfg(feature = "rustls-tls")]
    RustlsTls(StreamOwned<ClientSession, TcpStream>),
    /// Can't be built
    None,
}

impl NetworkStream {
    fn new(inner: InnerNetworkStream) -> Self {
        if let InnerNetworkStream::None = inner {
            debug_assert!(false, "InnerNetworkStream::None must never be built");
        }

        NetworkStream { inner }
    }

    /// Returns peer's address
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        match self.inner {
            InnerNetworkStream::Tcp(ref s) => s.peer_addr(),
            #[cfg(feature = "native-tls")]
            InnerNetworkStream::NativeTls(ref s) => s.get_ref().peer_addr(),
            #[cfg(feature = "rustls-tls")]
            InnerNetworkStream::RustlsTls(ref s) => s.get_ref().peer_addr(),
            InnerNetworkStream::None => {
                debug_assert!(false, "InnerNetworkStream::None must never be built");
                Ok(SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, 0, 1),
                    80,
                )))
            }
        }
    }

    /// Shutdowns the connection
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match self.inner {
            InnerNetworkStream::Tcp(ref s) => s.shutdown(how),
            #[cfg(feature = "native-tls")]
            InnerNetworkStream::NativeTls(ref s) => s.get_ref().shutdown(how),
            #[cfg(feature = "rustls-tls")]
            InnerNetworkStream::RustlsTls(ref s) => s.get_ref().shutdown(how),
            InnerNetworkStream::None => {
                debug_assert!(false, "InnerNetworkStream::None must never be built");
                Ok(())
            }
        }
    }

    pub fn connect<T: ToSocketAddrs>(
        server: T,
        timeout: Option<Duration>,
        tls_parameters: Option<&TlsParameters>,
    ) -> Result<NetworkStream, Error> {
        fn try_connect_timeout<T: ToSocketAddrs>(
            server: T,
            timeout: Duration,
        ) -> Result<TcpStream, Error> {
            let addrs = server.to_socket_addrs().map_err(error::connection)?;
            for addr in addrs {
                if let Ok(result) = TcpStream::connect_timeout(&addr, timeout) {
                    return Ok(result);
                }
            }
            Err(error::connection("Could not connect"))
        }

        let tcp_stream = match timeout {
            Some(t) => try_connect_timeout(server, t)?,
            None => TcpStream::connect(server).map_err(error::connection)?,
        };

        let mut stream = NetworkStream::new(InnerNetworkStream::Tcp(tcp_stream));
        if let Some(tls_parameters) = tls_parameters {
            stream.upgrade_tls(tls_parameters)?;
        }
        Ok(stream)
    }

    pub fn upgrade_tls(&mut self, tls_parameters: &TlsParameters) -> Result<(), Error> {
        match &self.inner {
            #[cfg(not(any(feature = "native-tls", feature = "rustls-tls")))]
            InnerNetworkStream::Tcp(_) => {
                let _ = tls_parameters;
                panic!("Trying to upgrade an NetworkStream without having enabled either the native-tls or the rustls-tls feature");
            }

            #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
            InnerNetworkStream::Tcp(_) => {
                // get owned TcpStream
                let tcp_stream = mem::replace(&mut self.inner, InnerNetworkStream::None);
                let tcp_stream = match tcp_stream {
                    InnerNetworkStream::Tcp(tcp_stream) => tcp_stream,
                    _ => unreachable!(),
                };

                self.inner = Self::upgrade_tls_impl(tcp_stream, tls_parameters)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    fn upgrade_tls_impl(
        tcp_stream: TcpStream,
        tls_parameters: &TlsParameters,
    ) -> Result<InnerNetworkStream, Error> {
        Ok(match &tls_parameters.connector {
            #[cfg(feature = "native-tls")]
            InnerTlsParameters::NativeTls(connector) => {
                let stream = connector
                    .connect(tls_parameters.domain(), tcp_stream)
                    .map_err(error::connection)?;
                InnerNetworkStream::NativeTls(stream)
            }
            #[cfg(feature = "rustls-tls")]
            InnerTlsParameters::RustlsTls(connector) => {
                use webpki::DNSNameRef;

                let domain = DNSNameRef::try_from_ascii_str(tls_parameters.domain())
                    .map_err(error::connection)?;
                let stream = StreamOwned::new(
                    ClientSession::new(&Arc::new(connector.clone()), domain),
                    tcp_stream,
                );

                InnerNetworkStream::RustlsTls(stream)
            }
        })
    }

    pub fn is_encrypted(&self) -> bool {
        match self.inner {
            InnerNetworkStream::Tcp(_) => false,
            #[cfg(feature = "native-tls")]
            InnerNetworkStream::NativeTls(_) => true,
            #[cfg(feature = "rustls-tls")]
            InnerNetworkStream::RustlsTls(_) => true,
            InnerNetworkStream::None => {
                debug_assert!(false, "InnerNetworkStream::None must never be built");
                false
            }
        }
    }

    pub fn set_read_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        match self.inner {
            InnerNetworkStream::Tcp(ref mut stream) => stream.set_read_timeout(duration),
            #[cfg(feature = "native-tls")]
            InnerNetworkStream::NativeTls(ref mut stream) => {
                stream.get_ref().set_read_timeout(duration)
            }
            #[cfg(feature = "rustls-tls")]
            InnerNetworkStream::RustlsTls(ref mut stream) => {
                stream.get_ref().set_read_timeout(duration)
            }
            InnerNetworkStream::None => {
                debug_assert!(false, "InnerNetworkStream::None must never be built");
                Ok(())
            }
        }
    }

    /// Set write timeout for IO calls
    pub fn set_write_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        match self.inner {
            InnerNetworkStream::Tcp(ref mut stream) => stream.set_write_timeout(duration),

            #[cfg(feature = "native-tls")]
            InnerNetworkStream::NativeTls(ref mut stream) => {
                stream.get_ref().set_write_timeout(duration)
            }
            #[cfg(feature = "rustls-tls")]
            InnerNetworkStream::RustlsTls(ref mut stream) => {
                stream.get_ref().set_write_timeout(duration)
            }

            InnerNetworkStream::None => {
                debug_assert!(false, "InnerNetworkStream::None must never be built");
                Ok(())
            }
        }
    }
}

impl Read for NetworkStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner {
            InnerNetworkStream::Tcp(ref mut s) => s.read(buf),
            #[cfg(feature = "native-tls")]
            InnerNetworkStream::NativeTls(ref mut s) => s.read(buf),
            #[cfg(feature = "rustls-tls")]
            InnerNetworkStream::RustlsTls(ref mut s) => s.read(buf),
            InnerNetworkStream::None => {
                debug_assert!(false, "InnerNetworkStream::None must never be built");
                Ok(0)
            }
        }
    }
}

impl Write for NetworkStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner {
            InnerNetworkStream::Tcp(ref mut s) => s.write(buf),
            #[cfg(feature = "native-tls")]
            InnerNetworkStream::NativeTls(ref mut s) => s.write(buf),
            #[cfg(feature = "rustls-tls")]
            InnerNetworkStream::RustlsTls(ref mut s) => s.write(buf),
            InnerNetworkStream::None => {
                debug_assert!(false, "InnerNetworkStream::None must never be built");
                Ok(0)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.inner {
            InnerNetworkStream::Tcp(ref mut s) => s.flush(),
            #[cfg(feature = "native-tls")]
            InnerNetworkStream::NativeTls(ref mut s) => s.flush(),
            #[cfg(feature = "rustls-tls")]
            InnerNetworkStream::RustlsTls(ref mut s) => s.flush(),
            InnerNetworkStream::None => {
                debug_assert!(false, "InnerNetworkStream::None must never be built");
                Ok(())
            }
        }
    }
}
