#[macro_use]
extern crate log;
extern crate env_logger;
extern crate smtp;

use std::sync::{Arc, Mutex};
use std::thread;

use smtp::sender::SenderBuilder;
use smtp::email::EmailBuilder;

fn main() {
    env_logger::init().unwrap();

    let sender = Arc::new(Mutex::new(SenderBuilder::localhost().unwrap().hello_name("localhost")
        .enable_connection_reuse(true).build()));

	let mut threads = Vec::new();
    for _ in 1..5 {

    	let th_sender = sender.clone();
    	threads.push(thread::spawn(move || {

        	let email = EmailBuilder::new()
                    	.to("user@localhost")
                    	.from("user@localhost")
                    	.body("Hello World!")
                    	.subject("Hello")
                    	.build();

    		let _ = th_sender.lock().unwrap().send(email);
		}));
    }

    for thread in threads {
    	let _ = thread.join();
    }

    let email = EmailBuilder::new()
                    .to("user@localhost")
                    .from("user@localhost")
                    .body("Hello World!")
                    .subject("Hello Bis")
                    .build();

	let mut sender = sender.lock().unwrap();
    let result = sender.send(email);
    sender.close();

    match result {
        Ok(..) => info!("Email sent successfully"),
        Err(error) => error!("{:?}", error),
    }
}
