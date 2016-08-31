extern crate lettre;

use lettre::transport::smtp::SmtpTransportBuilder;
use lettre::transport::EmailTransport;
use lettre::email::EmailBuilder;

#[test]
fn smtp_transport_simple() {
    let mut sender = SmtpTransportBuilder::localhost().unwrap().build();
    let email = EmailBuilder::new()
                    .to("root@localhost")
                    .from("user@localhost")
                    .body("Hello World!")
                    .subject("Hello")
                    .build()
                    .unwrap();
    let result = sender.send(email);
    assert!(result.is_ok());
}
