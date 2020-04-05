//! A trait to represent a stream

use crate::transport::smtp::{client::mock::MockStream, error::Error};
#[cfg(feature = "native-tls")]
use native_tls::{TlsConnector, TlsStream};
#[cfg(feature = "rustls")]
use rustls::{ClientConfig, ClientSession};
#[cfg(feature = "native-tls")]
use std::io::ErrorKind;
#[cfg(feature = "rustls")]
use std::sync::Arc;
use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpStream},
    time::Duration,
};

/// Parameters to use for secure clients
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct ClientTlsParameters {
    /// A connector from `native-tls`
    #[cfg(feature = "native-tls")]
    connector: TlsConnector,
    /// A client from `rustls`
    #[cfg(feature = "rustls")]
    // TODO use the same in all transports of the client
    connector: Box<ClientConfig>,
    /// The domain name which is expected in the TLS certificate from the server
    domain: String,
}

impl ClientTlsParameters {
    /// Creates a `ClientTlsParameters`
    #[cfg(feature = "native-tls")]
    pub fn new(domain: String, connector: TlsConnector) -> Self {
        ClientTlsParameters { connector, domain }
    }

    /// Creates a `ClientTlsParameters`
    #[cfg(feature = "rustls")]
    pub fn new(domain: String, connector: ClientConfig) -> Self {
        ClientTlsParameters {
            connector: Box::new(connector),
            domain,
        }
    }
}

/// Represents the different types of underlying network streams
pub enum NetworkStream {
    /// Plain TCP stream
    Tcp(TcpStream),
    /// Encrypted TCP stream
    #[cfg(feature = "native-tls")]
    Tls(Box<TlsStream<TcpStream>>),
    #[cfg(feature = "rustls")]
    Tls(Box<rustls::StreamOwned<ClientSession, TcpStream>>),
    /// Mock stream
    Mock(MockStream),
}

impl NetworkStream {
    /// Returns peer's address
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        match *self {
            NetworkStream::Tcp(ref s) => s.peer_addr(),
            #[cfg(feature = "native-tls")]
            NetworkStream::Tls(ref s) => s.get_ref().peer_addr(),
            #[cfg(feature = "rustls")]
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
            #[cfg(feature = "native-tls")]
            NetworkStream::Tls(ref s) => s.get_ref().shutdown(how),
            #[cfg(feature = "rustls")]
            NetworkStream::Tls(ref s) => s.get_ref().shutdown(how),
            NetworkStream::Mock(_) => Ok(()),
        }
    }
}

impl Read for NetworkStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.read(buf),
            #[cfg(feature = "native-tls")]
            NetworkStream::Tls(ref mut s) => s.read(buf),
            #[cfg(feature = "rustls")]
            NetworkStream::Tls(ref mut s) => s.read(buf),
            NetworkStream::Mock(ref mut s) => s.read(buf),
        }
    }
}

impl Write for NetworkStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.write(buf),
            #[cfg(feature = "native-tls")]
            NetworkStream::Tls(ref mut s) => s.write(buf),
            #[cfg(feature = "rustls")]
            NetworkStream::Tls(ref mut s) => s.write(buf),
            NetworkStream::Mock(ref mut s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.flush(),
            #[cfg(feature = "native-tls")]
            NetworkStream::Tls(ref mut s) => s.flush(),
            #[cfg(feature = "rustls")]
            NetworkStream::Tls(ref mut s) => s.flush(),
            NetworkStream::Mock(ref mut s) => s.flush(),
        }
    }
}

/// A trait for the concept of opening a stream
pub trait Connector: Sized {
    /// Opens a connection to the given IP socket
    fn connect(
        addr: &SocketAddr,
        timeout: Option<Duration>,
        tls_parameters: Option<&ClientTlsParameters>,
    ) -> Result<Self, Error>;
    /// Upgrades to TLS connection
    fn upgrade_tls(&mut self, tls_parameters: &ClientTlsParameters) -> Result<(), Error>;
    /// Is the NetworkStream encrypted
    fn is_encrypted(&self) -> bool;
}

impl Connector for NetworkStream {
    fn connect(
        addr: &SocketAddr,
        timeout: Option<Duration>,
        tls_parameters: Option<&ClientTlsParameters>,
    ) -> Result<NetworkStream, Error> {
        let tcp_stream = match timeout {
            Some(duration) => TcpStream::connect_timeout(addr, duration)?,
            None => TcpStream::connect(addr)?,
        };

        match tls_parameters {
            #[cfg(feature = "native-tls")]
            Some(context) => context
                .connector
                .connect(context.domain.as_ref(), tcp_stream)
                .map(|tls| NetworkStream::Tls(Box::new(tls)))
                .map_err(|e| Error::Io(io::Error::new(ErrorKind::Other, e))),
            #[cfg(feature = "rustls")]
            Some(context) => {
                let domain = webpki::DNSNameRef::try_from_ascii_str(&context.domain)?;

                Ok(NetworkStream::Tls(Box::new(rustls::StreamOwned::new(
                    ClientSession::new(&Arc::new(*context.connector.clone()), domain),
                    tcp_stream,
                ))))
            }
            None => Ok(NetworkStream::Tcp(tcp_stream)),
        }
    }

    fn upgrade_tls(&mut self, tls_parameters: &ClientTlsParameters) -> Result<(), Error> {
        *self = match *self {
            #[cfg(feature = "native-tls")]
            NetworkStream::Tcp(ref mut stream) => match tls_parameters
                .connector
                .connect(tls_parameters.domain.as_ref(), stream.try_clone().unwrap())
            {
                Ok(tls_stream) => NetworkStream::Tls(Box::new(tls_stream)),
                Err(err) => return Err(Error::Io(io::Error::new(ErrorKind::Other, err))),
            },
            #[cfg(feature = "rustls")]
            NetworkStream::Tcp(ref mut stream) => {
                let domain = webpki::DNSNameRef::try_from_ascii_str(&tls_parameters.domain)?;

                NetworkStream::Tls(Box::new(rustls::StreamOwned::new(
                    ClientSession::new(&Arc::new(*tls_parameters.connector.clone()), domain),
                    stream.try_clone().unwrap(),
                )))
            }
            NetworkStream::Tls(_) | NetworkStream::Mock(_) => return Ok(()),
        };

        Ok(())
    }

    fn is_encrypted(&self) -> bool {
        match *self {
            NetworkStream::Tcp(_) | NetworkStream::Mock(_) => false,
            NetworkStream::Tls(_) => true,
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
