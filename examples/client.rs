// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(default_type_params)]
#[macro_use] extern crate log;

extern crate smtp;
extern crate getopts;

use std::old_io::stdin;
use std::string::String;
use std::old_io::net::ip::Port;
use std::os;
use getopts::{optopt, optflag, getopts, OptGroup, usage};

use smtp::client::Client;
use smtp::error::SmtpResult;
use smtp::mailer::Email;

fn sendmail(source_address: &str, recipient_addresses: &[&str], message: &str, subject: &str,
        server: &str, port: Port, my_hostname: &str) -> SmtpResult {

    let mut email = Email::new();
    for destination in recipient_addresses.iter() {
        email.to(*destination);
    }
    email.from(source_address);
    email.body(message);
    email.subject(subject);
    email.date_now();

    let mut client: Client = Client::new((server, port));
    client.set_hello_name(my_hostname);
    client.send(email)
}

fn print_usage(description: String, _opts: &[OptGroup]) {
    println!("{}", usage(description.as_slice(), _opts));
}

fn main() {
    let args = os::args();

    let mut args_string = Vec::new();
    for arg in args.iter() {
        args_string.push(arg.clone());
    };

    let program = args[0].clone();
    let description = format!("Usage: {0} [options...] recipients\n\n\
                               This program reads a message on standard input until it reaches\
                               EOF, then tries to send it using the given paramters.\n\n\
                               Example: {0} -r user@example.org user@example.com < message.txt",
                              program);

    let opts = [
        optopt("s", "subject", "set the email subject", "SUBJECT"),
        optopt("r", "reverse-path", "set the sender address", "FROM_ADDRESS"),
        optopt("p", "port", "set the port to use, default is 25", "PORT"),
        optopt("a", "server", "set the server to use, default is localhost", "SERVER"),
        optopt("m", "my-hostname", "set the hostname used by the client", "MY_HOSTNAME"),
        optflag("h", "help", "print this help menu"),
    ];

    let matches = match getopts(args_string.tail(), &opts) {
        Ok(m) => m,
        Err(f) => panic!("{}", f),
    };

    if matches.opt_present("h") {
        print_usage(description, &opts);
        return;
    }

    if !matches.opt_present("r") {
        println!("The sender option is required");
        print_usage(program, &opts);
        return;
    }

    let recipients_str: &str = if !matches.free.is_empty() {
        matches.free[0].as_slice()
    } else {
        print_usage(description, &opts);
        return;
    };

    let mut recipients = Vec::new();
    for recipient in recipients_str.split(' ') {
        recipients.push(recipient);
    }

    let mut message = String::new();

    let mut line = stdin().read_line();
    while line.is_ok() {
        message.push_str(line.unwrap().as_slice());
        line = stdin().read_line();
    }

    match sendmail(
        // sender
        matches.opt_str("r").unwrap().as_slice(),
        // recipients
        recipients.as_slice(),
        // message content
        message.as_slice(),
        // subject
        match matches.opt_str("s") {
            Some(ref subject) => subject.as_slice(),
            None => "(empty subject)"
        },
        // server
        match matches.opt_str("a") {
            Some(ref server) => server.as_slice(),
            None => "localhost"
        },
        // port
        match matches.opt_str("p") {
            Some(port) => port.as_slice().parse::<Port>().unwrap(),
            None => 25
        },
        // my hostname
        match matches.opt_str("m") {
            Some(ref my_hostname) => my_hostname.as_slice(),
            None => "localhost"
        },
    )
    {
        Ok(..) => info!("Email sent successfully"),
        Err(error) => error!("{}", error),
    }
}
