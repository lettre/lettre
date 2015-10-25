extern crate lettre;

use lettre::transport::stub::StubEmailTransport;
use lettre::transport::EmailTransport;
use lettre::email::EmailBuilder;

#[test]
fn stub_transport() {
    let mut sender = StubEmailTransport;
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
