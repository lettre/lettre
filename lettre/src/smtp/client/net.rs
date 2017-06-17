//! A trait to represent a stream

use openssl::ssl::{Ssl, SslContext, SslStream};

use smtp::client::mock::MockStream;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpStream};
use std::time::Duration;

#[derive(Debug)]
/// Represents the different types of underlying network streams
pub enum NetworkStream {
    /// Plain TCP stream
    Tcp(TcpStream),
    /// Encrypted TCP stream
    Ssl(SslStream<TcpStream>),
    /// Mock stream
    Mock(MockStream),
}

impl NetworkStream {
    /// Returns peer's address
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        match *self {
            NetworkStream::Tcp(ref s) => s.peer_addr(),
            NetworkStream::Ssl(ref s) => s.get_ref().peer_addr(),
            NetworkStream::Mock(_) => {
                Ok(SocketAddr::V4(
                    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 80),
                ))
            }
        }
    }

    /// Shutdowns the connection
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref s) => s.shutdown(how),
            NetworkStream::Ssl(ref s) => s.get_ref().shutdown(how),
            NetworkStream::Mock(_) => Ok(()),
        }
    }
}

impl Read for NetworkStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.read(buf),
            NetworkStream::Ssl(ref mut s) => s.read(buf),
            NetworkStream::Mock(ref mut s) => s.read(buf),
        }
    }
}

impl Write for NetworkStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.write(buf),
            NetworkStream::Ssl(ref mut s) => s.write(buf),
            NetworkStream::Mock(ref mut s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.flush(),
            NetworkStream::Ssl(ref mut s) => s.flush(),
            NetworkStream::Mock(ref mut s) => s.flush(),
        }
    }
}

/// A trait for the concept of opening a stream
pub trait Connector: Sized {
    /// Opens a connection to the given IP socket
    fn connect(addr: &SocketAddr, ssl_context: Option<&SslContext>) -> io::Result<Self>;
    /// Upgrades to TLS connection
    fn upgrade_tls(&mut self, ssl_context: &SslContext) -> io::Result<()>;
    /// Is the NetworkStream encrypted
    fn is_encrypted(&self) -> bool;
}

impl Connector for NetworkStream {
    fn connect(addr: &SocketAddr, ssl_context: Option<&SslContext>) -> io::Result<NetworkStream> {
        let tcp_stream = try!(TcpStream::connect(addr));

        match ssl_context {
            Some(context) => {
                match Ssl::new(context) {
                    Ok(ssl) => {
                        ssl.connect(tcp_stream).map(NetworkStream::Ssl).map_err(
                            |e| {
                                io::Error::new(ErrorKind::Other, e)
                            },
                        )
                    }
                    Err(e) => Err(io::Error::new(ErrorKind::Other, e)),
                }
            }
            None => Ok(NetworkStream::Tcp(tcp_stream)),
        }
    }

    fn upgrade_tls(&mut self, ssl_context: &SslContext) -> io::Result<()> {

        *self = match *self {
            NetworkStream::Tcp(ref mut stream) => {
                match Ssl::new(ssl_context) {
                    Ok(ssl) => {
                        match ssl.connect(stream.try_clone().unwrap()) {
                            Ok(ssl_stream) => NetworkStream::Ssl(ssl_stream),
                            Err(err) => return Err(io::Error::new(ErrorKind::Other, err)),
                        }
                    }
                    Err(e) => return Err(io::Error::new(ErrorKind::Other, e)),
                }
            }
            NetworkStream::Ssl(_) => return Ok(()),
            NetworkStream::Mock(_) => return Ok(()),
        };

        Ok(())

    }

    fn is_encrypted(&self) -> bool {
        match *self {
            NetworkStream::Tcp(_) => false,
            NetworkStream::Ssl(_) => true,
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
            NetworkStream::Ssl(ref mut stream) => stream.get_ref().set_read_timeout(duration),
            NetworkStream::Mock(_) => Ok(()),
        }
    }

    /// Set write tiemout for IO calls
    fn set_write_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref mut stream) => stream.set_write_timeout(duration),
            NetworkStream::Ssl(ref mut stream) => stream.get_ref().set_write_timeout(duration),
            NetworkStream::Mock(_) => Ok(()),
        }
    }
}
