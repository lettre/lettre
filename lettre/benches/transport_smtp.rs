#![feature(test)]

extern crate lettre;
extern crate test;

use lettre::smtp::SmtpTransportBuilder;
use lettre::{EmailTransport, SimpleSendableEmail};

#[bench]
fn bench_simple_send(b: &mut test::Bencher) {
    let mut sender = SmtpTransportBuilder::new("127.0.0.1:2525").unwrap().build();
    b.iter(|| {
        let email = SimpleSendableEmail::new("user@localhost",
                                         vec!["root@localhost"],
                                         "id",
                                         "Hello world");
        let result = sender.send(email);
        assert!(result.is_ok());
    });
}

#[bench]
fn bench_reuse_send(b: &mut test::Bencher) {
    let mut sender = SmtpTransportBuilder::new("127.0.0.1:2525")
                         .unwrap()
                         .connection_reuse(true)
                         .build();
    b.iter(|| {
        let email = SimpleSendableEmail::new("user@localhost",
                                         vec!["root@localhost"],
                                         "file_id",
                                         "Hello file");
        let result = sender.send(email);
        assert!(result.is_ok());
    });
    sender.close()
}
