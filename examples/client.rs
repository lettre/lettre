#[macro_use]
extern crate log;
extern crate env_logger;
extern crate lettre;

use std::sync::{Arc, Mutex};
use std::thread;

use lettre::transport::smtp::SmtpTransportBuilder;
use lettre::transport::EmailTransport;
use lettre::mailer::Mailer;
use lettre::email::EmailBuilder;

fn main() {
    env_logger::init().unwrap();

    let sender = SmtpTransportBuilder::localhost().unwrap().hello_name("localhost")
        .connection_reuse(true).build();
    let mailer = Arc::new(Mutex::new(Mailer::new(sender)));

	let mut threads = Vec::new();
    for _ in 1..5 {

    	let th_mailer = mailer.clone();
    	threads.push(thread::spawn(move || {

        	let email = EmailBuilder::new()
                    	.to("user@localhost")
                    	.from("user@localhost")
                    	.body("Hello World!")
                    	.subject("Hello")
                    	.build().unwrap();

    		let _ = th_mailer.lock().unwrap().send(email);
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
                    .build().unwrap();

	let mut mailer = mailer.lock().unwrap();
    let result = mailer.send(email);
    mailer.close();

    match result {
        Ok(..) => info!("Email sent successfully"),
        Err(error) => error!("{:?}", error),
    }
}
