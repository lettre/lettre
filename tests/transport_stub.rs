extern crate lettre;

use lettre::email::EmailBuilder;
use lettre::transport::EmailTransport;
use lettre::transport::stub::StubEmailTransport;

#[test]
fn stub_transport() {
    let mut sender = StubEmailTransport;
    let email = EmailBuilder::new()
        .to("root@localhost")
        .from("user@localhost")
        .body("Hello World!")
        .subject("Hello stub")
        .build()
        .unwrap();
    let result = sender.send(email);
    assert!(result.is_ok());
}
