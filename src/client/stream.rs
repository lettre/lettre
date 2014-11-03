//! TODO

use response::Response;
use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use std::str::from_utf8;
use std::vec::Vec;

static BUFFER_SIZE: uint = 1024;

/// TODO
pub trait ClientStream {
    /// TODO
    fn send_and_get_response(&mut self, string: &str) -> Response;
    /// TODO
    fn get_reply(&mut self) -> Option<Response>;
    /// TODO
    fn read_into_string(&mut self) -> IoResult<String>;
}

impl ClientStream for TcpStream {
    /// Sends a complete message or a command to the server and get the response
    fn send_and_get_response(&mut self, string: &str) -> Response {
        match self.write_str(format!("{}", string).as_slice()) {
            Ok(..)  => debug!("Wrote: {}", string),
            Err(..) => panic!("Could not write to stream")
        }

        match self.get_reply() {
            Some(response) => {debug!("Read: {}", response); response},
            None           => panic!("No answer")
        }
    }

    /// Reads on the stream into a string
    fn read_into_string(&mut self) -> IoResult<String> {
        let mut more = true;
        let mut result = String::new();
        // TODO: Set appropriate timeouts
        self.set_timeout(Some(1000));

        while more {
            let mut buf: Vec<u8> = Vec::with_capacity(BUFFER_SIZE);
            let response = match self.push(BUFFER_SIZE, &mut buf) {
                Ok(bytes_read) => {
                    more = bytes_read == BUFFER_SIZE;
                    if bytes_read > 0 {
                        from_utf8(buf.slice_to(bytes_read)).unwrap()
                    } else {
                        ""
                    }
                },
                // TODO: Manage error kinds
                Err(..) => {more = false; ""},
            };
            result.push_str(response);
        }
        debug!("Read: {}", result);
        return Ok(result);
    }

    /// Gets the SMTP response
    fn get_reply(&mut self) -> Option<Response> {
        let response = match self.read_into_string() {
            Ok(string) => string,
            Err(..)    => panic!("No answer")
        };
        from_str::<Response>(response.as_slice())
    }
}
