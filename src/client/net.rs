//! A trait to represent a stream

use std::io;
use std::net::SocketAddr;
use std::net::TcpStream;

/// A trait for the concept of opening a stream
pub trait Connector {
    /// Opens a connection to the given IP socket
    fn connect(addr: &SocketAddr) -> io::Result<Self>;
}

impl Connector for SmtpStream {
    fn connect(addr: &SocketAddr) -> io::Result<SmtpStream> {
        TcpStream::connect(addr)
    }
}

/// Represents an atual SMTP network stream
//Used later for ssl
pub type SmtpStream = TcpStream;


