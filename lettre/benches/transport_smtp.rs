#![feature(test)]

extern crate lettre;
extern crate test;

use lettre::{ClientSecurity, SmtpTransport};
use lettre::{EmailAddress, EmailTransport, SimpleSendableEmail};
use lettre::smtp::ConnectionReuseParameters;

#[bench]
fn bench_simple_send(b: &mut test::Bencher) {
    let mut sender = SmtpTransport::builder("127.0.0.1:2525", ClientSecurity::None).unwrap()
                                                                                   .build();
    b.iter(|| {
               let email =
            SimpleSendableEmail::new(EmailAddress::new("user@localhost".to_string()),
                                     vec![EmailAddress::new("root@localhost".to_string())],
                                     "id".to_string(),
                                     "Hello world".to_string());
               let result = sender.send(&email);
               assert!(result.is_ok());
           });
}

#[bench]
fn bench_reuse_send(b: &mut test::Bencher) {
    let mut sender = SmtpTransport::builder("127.0.0.1:2525", ClientSecurity::None)
        .unwrap()
        .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
        .build();
    b.iter(|| {
               let email =
            SimpleSendableEmail::new(EmailAddress::new("user@localhost".to_string()),
                                     vec![EmailAddress::new("root@localhost".to_string())],
                                     "id".to_string(),
                                     "Hello world".to_string());
               let result = sender.send(&email);
               assert!(result.is_ok());
           });
    sender.close()
}
