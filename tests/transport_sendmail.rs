extern crate lettre;

use lettre::transport::sendmail::SendmailTransport;
use lettre::transport::EmailTransport;
use lettre::email::EmailBuilder;

#[test]
fn sendmail_transport_simple() {
    let mut sender = SendmailTransport;
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
