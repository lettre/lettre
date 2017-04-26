extern crate lettre;

use lettre::email::EmailBuilder;
use lettre::transport::EmailTransport;
use lettre::transport::smtp::SmtpTransportBuilder;

#[test]
fn smtp_transport_simple() {
    let mut sender = SmtpTransportBuilder::localhost().unwrap().build();
    let email = EmailBuilder::new()
        .to("root@localhost")
        .from("user@localhost")
        .body("Hello World!")
        .subject("Hello smtp")
        .build()
        .unwrap();
    let result = sender.send(email);
    assert!(result.is_ok());
}
