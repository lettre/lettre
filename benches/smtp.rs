#![feature(test)]

extern crate lettre;
extern crate test;

use lettre::transport::smtp::SmtpTransportBuilder;
use lettre::transport::EmailTransport;
use lettre::mailer::Mailer;
use lettre::email::EmailBuilder;

#[bench]
fn bench_simple_send(b: &mut test::Bencher) {
    let sender = SmtpTransportBuilder::new("127.0.0.1:2525").unwrap().build();
    let mut mailer = Mailer::new(sender);
    b.iter(|| {
        let email = EmailBuilder::new()
                        .to("root@localhost")
                        .from("user@localhost")
                        .body("Hello World!")
                        .subject("Hello")
                        .build()
                        .unwrap();
        let result = mailer.send(email);
        assert!(result.is_ok());
    });
}

#[bench]
fn bench_reuse_send(b: &mut test::Bencher) {
    let sender = SmtpTransportBuilder::new("127.0.0.1:2525")
                     .unwrap()
                     .connection_reuse(true)
                     .build();
    let mut mailer = Mailer::new(sender);
    b.iter(|| {
        let email = EmailBuilder::new()
                        .to("root@localhost")
                        .from("user@localhost")
                        .body("Hello World!")
                        .subject("Hello")
                        .build()
                        .unwrap();
        let result = mailer.send(email);
        assert!(result.is_ok());
    });
    mailer.close()
}
