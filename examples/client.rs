// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate smtp;

use smtp::sender::{Sender, SenderBuilder};
use smtp::mailer::EmailBuilder;

fn main() {
    env_logger::init().unwrap();

    let email = EmailBuilder::new()
                    .to("user@localhost")
                    .from("user@localhost")
                    .body("Hello World!")
                    .subject("Hello")
                    .build();

    let mut sender: Sender = SenderBuilder::localhost().hello_name("localhost")
        .enable_connection_reuse(true).build();

    for _ in (1..5) {
        let _ = sender.send(email.clone());
    }
    let result = sender.send(email);
    sender.close();

    match result {
        Ok(..) => info!("Email sent successfully"),
        Err(error) => error!("{:?}", error),
    }
}
