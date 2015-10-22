extern crate lettre;

use std::sync::{Arc, Mutex};
use std::thread;

use lettre::transport::smtp::SmtpTransportBuilder;
use lettre::transport::EmailTransport;
use lettre::mailer::Mailer;
use lettre::email::EmailBuilder;

#[test]
fn simple_sender() {
    let sender = SmtpTransportBuilder::localhost().unwrap().build();
    let mut mailer = Mailer::new(sender);
    let email = EmailBuilder::new()
                    .to("root@localhost")
                    .from("user@localhost")
                    .body("Hello World!")
                    .subject("Hello")
                    .build()
                    .unwrap();
    let result = mailer.send(email);
    assert!(result.is_ok());
}

#[test]
fn multithreaded_sender() {
    let sender = SmtpTransportBuilder::localhost()
                     .unwrap()
                     .hello_name("localhost")
                     .connection_reuse(true)
                     .build();
    let mailer = Arc::new(Mutex::new(Mailer::new(sender)));

    let mut threads = Vec::new();
    for _ in 1..5 {

        let th_mailer = mailer.clone();
        threads.push(thread::spawn(move || {

            let email = EmailBuilder::new()
                            .to("root@localhost")
                            .from("user@localhost")
                            .body("Hello World!")
                            .subject("Hello")
                            .build()
                            .unwrap();

            let result = th_mailer.lock().unwrap().send(email);
            assert!(result.is_ok());
        }));
    }

    for thread in threads {
        let _ = thread.join();
    }

    let email = EmailBuilder::new()
                    .to("root@localhost")
                    .from("user@localhost")
                    .body("Hello World!")
                    .subject("Hello Bis")
                    .build()
                    .unwrap();

    let mut mailer = mailer.lock().unwrap();
    let final_result = mailer.send(email);
    mailer.close();

    assert!(final_result.is_ok());
}
