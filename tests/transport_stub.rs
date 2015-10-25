extern crate lettre;

use lettre::transport::stub::StubEmailTransport;
use lettre::transport::EmailTransport;
use lettre::mailer::Mailer;
use lettre::email::EmailBuilder;

#[test]
fn stub_transport() {
    let sender = StubEmailTransport;
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
