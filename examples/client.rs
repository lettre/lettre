// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(core, old_io, net, rustc_private, collections)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate smtp;
extern crate getopts;

use std::old_io::stdin;
use std::string::String;
use std::env;
use getopts::{optopt, optflag, getopts, OptGroup, usage};
use std::net::TcpStream;

use smtp::sender::{Sender, SenderBuilder};
use smtp::error::SmtpResult;
use smtp::mailer::EmailBuilder;

fn sendmail(source_address: String, recipient_addresses: Vec<String>, message: String, subject: String,
        server: String, port: u16, my_hostname: String, number: u16) -> SmtpResult {

    let mut email_builder = EmailBuilder::new();
    for destination in recipient_addresses.iter() {
        email_builder = email_builder.to(destination.as_slice());
    }
    let email = email_builder.from(source_address.as_slice())
                         .body(message.as_slice())
                         .subject(subject.as_slice())
                         .build();

    let mut sender: Sender<TcpStream> = SenderBuilder::new((server.as_slice(), port)).hello_name(my_hostname.as_slice())
        .enable_connection_reuse(true).build();

    for _ in range(1, number) {
        let _ = sender.send(email.clone());
    }
    let result = sender.send(email);
    sender.close();

    result
}

fn print_usage(description: String, _opts: &[OptGroup]) {
    println!("{}", usage(description.as_slice(), _opts));
}

fn main() {
    env_logger::init().unwrap();

    let args = env::args();

    let mut args_string = Vec::new();
    for arg in args {
        args_string.push(arg.clone());
    };

    let program = args_string[0].clone();
    let description = format!("Usage: {0} [options...] recipients\n\n\
                               This program reads a message on standard input until it reaches\
                               EOF, then tries to send it using the given paramters.\n\n\
                               Example: {0} -r user@example.org user@example.com < message.txt",
                              program);

    let opts = [
        optopt("n", "number", "set the number of emails to send", "NUMBER"),
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
        recipients.push(recipient.to_string());
    }

    let mut message = String::new();

    let mut line = stdin().read_line();
    while line.is_ok() {
        message.push_str(line.unwrap().as_slice());
        line = stdin().read_line();
    }

    match sendmail(
        // sender
        matches.opt_str("r").unwrap().clone(),
        // recipients
        recipients,
        // message content
        message,
        // subject
        match matches.opt_str("s") {
            Some(ref subject) => subject.clone(),
            None => "(empty subject)".to_string(),
        },
        // server
        match matches.opt_str("a") {
            Some(ref server) => server.clone(),
            None => "localhost".to_string(),
        },
        // port
        match matches.opt_str("p") {
            Some(port) => port.as_slice().parse::<u16>().unwrap(),
            None => 25,
        },
        // my hostname
        match matches.opt_str("m") {
            Some(ref my_hostname) => my_hostname.clone(),
            None => "localhost".to_string(),
        },
        // number of copies
        match matches.opt_str("n") {
            Some(ref n) => n.as_slice().parse::<u16>().unwrap(),
            None => 1,
        },
    )
    {
        Ok(..) => info!("Email sent successfully"),
        Err(error) => error!("{}", error),
    }
}
