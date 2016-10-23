extern crate lettre;

use lettre::email::EmailBuilder;
use lettre::transport::EmailTransport;
use lettre::transport::sendmail::SendmailTransport;

#[test]
fn sendmail_transport_simple() {
    let mut sender = SendmailTransport::new();
    let email = EmailBuilder::new()
        .to("root@localhost")
        .from("user@localhost")
        .body("Hello World!")
        .subject("Hello sendmail")
        .build()
        .unwrap();
    let result = sender.send(email);
    println!("{:?}", result);
    assert!(result.is_ok());
}
