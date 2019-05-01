//! A trait to represent a stream

use crate::smtp::client::mock::MockStream;
use native_tls::{Protocol, TlsConnector, TlsStream};
use std::io::{self, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpStream};
use std::time::Duration;

/// Parameters to use for secure clients
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct ClientTlsParameters {
    /// A connector from `native-tls`
    pub connector: TlsConnector,
    /// The domain to send during the TLS handshake
    pub domain: String,
}

impl ClientTlsParameters {
    /// Creates a `ClientTlsParameters`
    pub fn new(domain: String, connector: TlsConnector) -> ClientTlsParameters {
        ClientTlsParameters { connector, domain }
    }
}

/// Accepted protocols by default.
/// This removes TLS 1.0 and 1.1 compared to tls-native defaults.
pub const DEFAULT_TLS_PROTOCOLS: &[Protocol] = &[Protocol::Tlsv12];

#[derive(Debug)]
/// Represents the different types of underlying network streams
pub enum NetworkStream {
    /// Plain TCP stream
    Tcp(TcpStream),
    /// Encrypted TCP stream
    Tls(TlsStream<TcpStream>),
    /// Mock stream
    Mock(MockStream),
}

impl NetworkStream {
    /// Returns peer's address
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        match *self {
            NetworkStream::Tcp(ref s) => s.peer_addr(),
            NetworkStream::Tls(ref s) => s.get_ref().peer_addr(),
            NetworkStream::Mock(_) => Ok(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(127, 0, 0, 1),
                80,
            ))),
        }
    }

    /// Shutdowns the connection
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref s) => s.shutdown(how),
            NetworkStream::Tls(ref s) => s.get_ref().shutdown(how),
            NetworkStream::Mock(_) => Ok(()),
        }
    }
}

impl Read for NetworkStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.read(buf),
            NetworkStream::Tls(ref mut s) => s.read(buf),
            NetworkStream::Mock(ref mut s) => s.read(buf),
        }
    }
}

impl Write for NetworkStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.write(buf),
            NetworkStream::Tls(ref mut s) => s.write(buf),
            NetworkStream::Mock(ref mut s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.flush(),
            NetworkStream::Tls(ref mut s) => s.flush(),
            NetworkStream::Mock(ref mut s) => s.flush(),
        }
    }
}

/// A trait for the concept of opening a stream
pub trait Connector: Sized {
    /// Opens a connection to the given IP socket
    fn connect(addr: &SocketAddr, timeout: Option<Duration>, tls_parameters: Option<&ClientTlsParameters>)
        -> io::Result<Self>;
    /// Upgrades to TLS connection
    fn upgrade_tls(&mut self, tls_parameters: &ClientTlsParameters) -> io::Result<()>;
    /// Is the NetworkStream encrypted
    fn is_encrypted(&self) -> bool;
}

impl Connector for NetworkStream {
    fn connect(
        addr: &SocketAddr,
        timeout: Option<Duration>,
        tls_parameters: Option<&ClientTlsParameters>,
    ) -> io::Result<NetworkStream> {
        let tcp_stream = match timeout {
            Some(duration) => TcpStream::connect_timeout(addr, duration)?,
            None => TcpStream::connect(addr)?,
        };

        match tls_parameters {
            Some(context) => context
                .connector
                .connect(context.domain.as_ref(), tcp_stream)
                .map(NetworkStream::Tls)
                .map_err(|e| io::Error::new(ErrorKind::Other, e)),
            None => Ok(NetworkStream::Tcp(tcp_stream)),
        }
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::match_same_arms))]
    fn upgrade_tls(&mut self, tls_parameters: &ClientTlsParameters) -> io::Result<()> {
        *self = match *self {
            NetworkStream::Tcp(ref mut stream) => match tls_parameters
                .connector
                .connect(tls_parameters.domain.as_ref(), stream.try_clone().unwrap())
            {
                Ok(tls_stream) => NetworkStream::Tls(tls_stream),
                Err(err) => return Err(io::Error::new(ErrorKind::Other, err)),
            },
            NetworkStream::Tls(_) => return Ok(()),
            NetworkStream::Mock(_) => return Ok(()),
        };

        Ok(())
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::match_same_arms))]
    fn is_encrypted(&self) -> bool {
        match *self {
            NetworkStream::Tcp(_) => false,
            NetworkStream::Tls(_) => true,
            NetworkStream::Mock(_) => false,
        }
    }
}

/// A trait for read and write timeout support
pub trait Timeout: Sized {
    /// Set read timeout for IO calls
    fn set_read_timeout(&mut self, duration: Option<Duration>) -> io::Result<()>;
    /// Set write timeout for IO calls
    fn set_write_timeout(&mut self, duration: Option<Duration>) -> io::Result<()>;
}

impl Timeout for NetworkStream {
    fn set_read_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref mut stream) => stream.set_read_timeout(duration),
            NetworkStream::Tls(ref mut stream) => stream.get_ref().set_read_timeout(duration),
            NetworkStream::Mock(_) => Ok(()),
        }
    }

    /// Set write timeout for IO calls
    fn set_write_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref mut stream) => stream.set_write_timeout(duration),
            NetworkStream::Tls(ref mut stream) => stream.get_ref().set_write_timeout(duration),
            NetworkStream::Mock(_) => Ok(()),
        }
    }
}
