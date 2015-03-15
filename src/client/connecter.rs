// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A trait to represent a connected stream

use std::io;
use std::net::SocketAddr;
use std::net::TcpStream;

/// A trait for the concept of opening a stream
pub trait Connecter {
    /// Opens a connection to the given IP socket
    fn connect(addr: &SocketAddr) -> io::Result<Self>;
}

impl Connecter for TcpStream {
    fn connect(addr: &SocketAddr) -> io::Result<TcpStream> {
        TcpStream::connect(addr)
    }
}
