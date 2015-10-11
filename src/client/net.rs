//! A trait to represent a stream

use std::io;
use std::io::{Read, Write, ErrorKind};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::fmt;
use std::fmt::{Debug, Formatter};
use openssl::ssl::{SslContext, SslStream};

/// A trait for the concept of opening a stream
pub trait Connector {
    /// Opens a connection to the given IP socket
    fn connect(addr: &SocketAddr, ssl_context: Option<&SslContext>) -> io::Result<Self>;
}

impl Connector for NetworkStream {
    fn connect(addr: &SocketAddr, ssl_context: Option<&SslContext>) -> io::Result<NetworkStream> {
        let tcp_stream = try!(TcpStream::connect(addr));

        match ssl_context {
            Some(context) => match SslStream::new(&context, tcp_stream) {
                Ok(stream) => Ok(NetworkStream::Ssl(stream)),
                Err(err) => Err(io::Error::new(ErrorKind::Other, err)),
            },
            None => Ok(NetworkStream::Plain(tcp_stream)),
        }
    }
}

/// TODO
pub enum NetworkStream {
    /// TODO
    Plain(TcpStream),
    /// TODO
    Ssl(SslStream<TcpStream>),
}

impl Clone for NetworkStream {
    #[inline]
    fn clone(&self) -> NetworkStream {
        match self {
            &NetworkStream::Plain(ref stream) => NetworkStream::Plain(stream.try_clone().unwrap()),
            &NetworkStream::Ssl(ref stream) => NetworkStream::Ssl(stream.try_clone().unwrap()),
        }
    }
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
