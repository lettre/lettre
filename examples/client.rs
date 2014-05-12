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
extern crate getopts;
use std::io::stdin;
use std::io::net::tcp::TcpStream;
use std::strbuf::StrBuf;
use std::io::net::ip::Port;
use std::os;
use smtp::client::SmtpClient;
use getopts::{optopt,optflag,getopts,OptGroup,usage};

fn sendmail(source_address: StrBuf, recipient_addresses: Vec<StrBuf>, message: StrBuf, server: StrBuf, port: Option<Port>, my_hostname: Option<StrBuf>) {
    let mut email_client: SmtpClient<StrBuf, TcpStream> =
        SmtpClient::new(
            server,
            port,
            my_hostname);
    email_client.send_mail(
            source_address,
            recipient_addresses,
            message
    );
}

fn print_usage(description: &str, _opts: &[OptGroup]) {
    println!("{}", usage(description, _opts));
}

fn main() {
    let args = os::args();

    let program = args.get(0).clone();
    let description = format!("Usage: {0} [options...] recipients\n\n\
                               This program reads a message on standard input until it reaches EOF,\
                               then tries to send it using the given paramters.\n\n\
                               Example: {0} -r user@example.org user@example.com < message.txt", program);

    let opts = [
        optopt("r", "reverse-path", "set the sender address", "FROM_ADDRESS"),
        optopt("p", "port", "set the port to use, default is 25", "PORT"),
        optopt("s", "server", "set the server to use, default is localhost", "SERVER"),
        optopt("m", "my-hostname", "set the hostname used by the client", "MY_HOSTNAME"),
        optflag("h", "help", "print this help menu"),
        optflag("v", "verbose", "display the transaction details"),
    ];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };
    if matches.opt_present("h") {
        print_usage(description, opts);
        return;
    }

    let sender = match matches.opt_str("r") {
        Some(sender) => StrBuf::from_str(sender),
        None         => {
            println!("The sender option is required");
            print_usage(program, opts);
            return;
        }
    };

    let server = match matches.opt_str("s") {
        Some(server) => StrBuf::from_str(server),
        None         => StrBuf::from_str("localhost")
    };

    let my_hostname = match matches.opt_str("m") {
        Some(my_hostname) => Some(StrBuf::from_str(my_hostname)),
        None         => None
    };

    let port = match matches.opt_str("p") {
        Some(port) => from_str::<Port>(port),
        None       => None

    };

    let recipients_str: &str = if !matches.free.is_empty() {
        (*matches.free.get(0)).clone()
    } else {
        print_usage(description, opts);
        return;
    };
    let mut recipients = Vec::new();
    for recipient in recipients_str.split(' ') {
        recipients.push(StrBuf::from_str(recipient))
    }

    let mut message = StrBuf::new();
    for line in stdin().lines() {
        message = message.append(line.unwrap().to_str());
    }

    sendmail(sender, recipients, message, server, port, my_hostname);
}
