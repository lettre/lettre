// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_id = "client"]

extern crate smtp;
use std::io::net::tcp::TcpStream;
use smtp::client::SmtpClient;
use std::strbuf::StrBuf;

fn main() {
    let mut email_client: SmtpClient<StrBuf, TcpStream> = SmtpClient::new(StrBuf::from_str("localhost"), None, None);
    email_client.send_mail(StrBuf::from_str("user@localhost"), vec!(StrBuf::from_str("user@localhost")), StrBuf::from_str("Test email"));
}
