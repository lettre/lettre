//! A trait to represent a stream

use openssl::ssl::{SslContext, SslStream};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

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
                match SslStream::connect(context, tcp_stream) {
                    Ok(stream) => Ok(NetworkStream::Ssl(stream)),
                    Err(err) => Err(io::Error::new(ErrorKind::Other, err)),
                }
            }
            None => Ok(NetworkStream::Plain(tcp_stream)),
        }
    }

    fn upgrade_tls(&mut self, ssl_context: &SslContext) -> io::Result<()> {

        *self = match *self {
            NetworkStream::Plain(ref mut stream) => {
                match SslStream::connect(ssl_context, stream.try_clone().unwrap()) {
                    Ok(ssl_stream) => NetworkStream::Ssl(ssl_stream),
                    Err(err) => return Err(io::Error::new(ErrorKind::Other, err)),
                }
            }
            NetworkStream::Ssl(_) => return Ok(()),
        };

        Ok(())

    }

    fn is_encrypted(&self) -> bool {
        match *self {
            NetworkStream::Plain(_) => false,
            NetworkStream::Ssl(_) => true,
        }
    }
}


/// Represents the different types of underlying network streams
pub enum NetworkStream {
    /// Plain TCP
    Plain(TcpStream),
    /// SSL over TCP
    Ssl(SslStream<TcpStream>),
}

impl Debug for NetworkStream {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("NetworkStream(_)")
    }
}

impl Read for NetworkStream {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            NetworkStream::Plain(ref mut stream) => stream.read(buf),
            NetworkStream::Ssl(ref mut stream) => stream.read(buf),
        }
    }
}

impl Write for NetworkStream {
    #[inline]
    fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
        match *self {
            NetworkStream::Plain(ref mut stream) => stream.write(msg),
            NetworkStream::Ssl(ref mut stream) => stream.write(msg),
        }
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        match *self {
            NetworkStream::Plain(ref mut stream) => stream.flush(),
            NetworkStream::Ssl(ref mut stream) => stream.flush(),
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
            NetworkStream::Plain(ref mut stream) => stream.set_read_timeout(duration),
            NetworkStream::Ssl(ref mut stream) => stream.get_mut().set_read_timeout(duration),
        }
    }

    /// Set write tiemout for IO calls
    fn set_write_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        match *self {
            NetworkStream::Plain(ref mut stream) => stream.set_write_timeout(duration),
            NetworkStream::Ssl(ref mut stream) => stream.get_mut().set_write_timeout(duration),
        }
    }
}