// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Taken fron rust-http

//! TODO

use std::io::IoResult;
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;

/// A trait for the concept of opening a stream connected to a IP socket address.
///
/// Why is this here? So that we can implement things which must make
/// connections in terms of *anything* that can make such a connection rather
/// than in terms of `TcpStream` only. This is handy for testing and for SSL.
pub trait Connecter {
    /// TODO
    fn connect(host: &str, port: u16) -> IoResult<Self>;
    /// TODO
    fn peer_name(&mut self) -> IoResult<SocketAddr>;
}

impl Connecter for TcpStream {
    fn connect(host: &str, port: u16) -> IoResult<TcpStream> {
        TcpStream::connect((host, port))
    }

    fn peer_name(&mut self) -> IoResult<SocketAddr> {
        self.peer_name()
    }
}
