//! A trait to represent a stream

use crate::transport::smtp::{client::mock::MockStream, error::Error};
#[cfg(feature = "native-tls")]
use native_tls::{TlsConnector, TlsStream};
#[cfg(feature = "rustls-tls")]
use rustls::{ClientConfig, ClientSession};
#[cfg(feature = "native-tls")]
use std::io::ErrorKind;
#[cfg(feature = "rustls-tls")]
use std::sync::Arc;
use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpStream, ToSocketAddrs},
    time::Duration,
};

/// Parameters to use for secure clients
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct TlsParameters {
    /// A connector from `native-tls`
    #[cfg(feature = "native-tls")]
    connector: TlsConnector,
    /// A client from `rustls`
    #[cfg(feature = "rustls-tls")]
    // TODO use the same in all transports of the client
    connector: Box<ClientConfig>,
    /// The domain name which is expected in the TLS certificate from the server
    domain: String,
}

impl TlsParameters {
    /// Creates a `TlsParameters`
    #[cfg(feature = "native-tls")]
    pub fn new(domain: String, connector: TlsConnector) -> Self {
        Self { connector, domain }
    }

    /// Creates a `TlsParameters`
    #[cfg(feature = "rustls-tls")]
    pub fn new(domain: String, connector: ClientConfig) -> Self {
        Self {
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
    #[cfg(feature = "rustls-tls")]
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
            #[cfg(feature = "rustls-tls")]
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
            #[cfg(feature = "rustls-tls")]
            NetworkStream::Tls(ref s) => s.get_ref().shutdown(how),
            NetworkStream::Mock(_) => Ok(()),
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
            let addrs = server.to_socket_addrs()?;
            for addr in addrs {
                let result = TcpStream::connect_timeout(&addr, timeout);
                if result.is_ok() {
                    return result.map_err(|e| e.into());
                }
            }
            Err(Error::Client("Could not connect"))
        }

        let tcp_stream = match timeout {
            Some(t) => try_connect_timeout(server, t)?,
            None => TcpStream::connect(server)?,
        };

        match tls_parameters {
            #[cfg(feature = "native-tls")]
            Some(context) => context
                .connector
                .connect(context.domain.as_ref(), tcp_stream)
                .map(|tls| NetworkStream::Tls(Box::new(tls)))
                .map_err(|e| Error::Io(io::Error::new(ErrorKind::Other, e))),
            #[cfg(feature = "rustls-tls")]
            Some(context) => {
                let domain = webpki::DNSNameRef::try_from_ascii_str(&context.domain)?;

                Ok(NetworkStream::Tls(Box::new(rustls::StreamOwned::new(
                    ClientSession::new(&Arc::new(*context.connector.clone()), domain),
                    tcp_stream,
                ))))
            }
            #[cfg(not(any(feature = "native-tls", feature = "rustls-tls")))]
            Some(_) => panic!("TLS configuration without support"),
            None => Ok(NetworkStream::Tcp(tcp_stream)),
        }
    }

    #[allow(unused_variables, unreachable_code)]
    pub fn upgrade_tls(&mut self, tls_parameters: &TlsParameters) -> Result<(), Error> {
        *self = match *self {
            #[cfg(feature = "native-tls")]
            NetworkStream::Tcp(ref mut stream) => match tls_parameters
                .connector
                .connect(tls_parameters.domain.as_ref(), stream.try_clone().unwrap())
            {
                Ok(tls_stream) => NetworkStream::Tls(Box::new(tls_stream)),
                Err(err) => return Err(Error::Io(io::Error::new(ErrorKind::Other, err))),
            },
            #[cfg(feature = "rustls-tls")]
            NetworkStream::Tcp(ref mut stream) => {
                let domain = webpki::DNSNameRef::try_from_ascii_str(&tls_parameters.domain)?;

                NetworkStream::Tls(Box::new(rustls::StreamOwned::new(
                    ClientSession::new(&Arc::new(*tls_parameters.connector.clone()), domain),
                    stream.try_clone().unwrap(),
                )))
            }
            #[cfg(not(any(feature = "native-tls", feature = "rustls-tls")))]
            NetworkStream::Tcp(_) => panic!("STARTTLS without TLS support"),
            #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
            NetworkStream::Tls(_) => return Ok(()),
            NetworkStream::Mock(_) => return Ok(()),
        };

        Ok(())
    }

    pub fn is_encrypted(&self) -> bool {
        match *self {
            NetworkStream::Tcp(_) | NetworkStream::Mock(_) => false,
            #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
            NetworkStream::Tls(_) => true,
        }
    }

    pub fn set_read_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref mut stream) => stream.set_read_timeout(duration),
            #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
            NetworkStream::Tls(ref mut stream) => stream.get_ref().set_read_timeout(duration),
            NetworkStream::Mock(_) => Ok(()),
        }
    }

    /// Set write timeout for IO calls
    pub fn set_write_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref mut stream) => stream.set_write_timeout(duration),
            #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
            NetworkStream::Tls(ref mut stream) => stream.get_ref().set_write_timeout(duration),
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
            #[cfg(feature = "rustls-tls")]
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
            #[cfg(feature = "rustls-tls")]
            NetworkStream::Tls(ref mut s) => s.write(buf),
            NetworkStream::Mock(ref mut s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.flush(),
            #[cfg(feature = "native-tls")]
            NetworkStream::Tls(ref mut s) => s.flush(),
            #[cfg(feature = "rustls-tls")]
            NetworkStream::Tls(ref mut s) => s.flush(),
            NetworkStream::Mock(ref mut s) => s.flush(),
        }
    }
}
