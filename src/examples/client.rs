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
use std::strbuf::StrBuf;
use std::io::net::ip::Port;
use std::os;
use smtp::client::SmtpClient;

fn main() {
    //! For now, only one word messages
    //!
    //! TODO: use parameters, flexible syntax
    let args = os::args();
    match args.len() {
        6 => sendmail(args[1], args[2], args[3], args[4], args[5]),
        _ => {
            println!("Usage: {} source_address recipient_address message server port", args[0]);
            return;
        },
    };
}

fn sendmail(source_address: &str, recipient_address: &str, message: &str, server: &str, port: &str) {
    let mut email_client: SmtpClient<StrBuf, TcpStream> = 
        SmtpClient::new(StrBuf::from_str(server), from_str::<Port>(port), None);
    email_client.send_mail(
            StrBuf::from_str(source_address),
            vec!(StrBuf::from_str(recipient_address)),
            StrBuf::from_str(message)
    );
}
