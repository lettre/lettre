extern crate lettre;

use lettre::{EmailTransport, SimpleSendableEmail};
use lettre::sendmail::SendmailTransport;

#[test]
fn sendmail_transport_simple() {
    let mut sender = SendmailTransport::new();
    let email = SimpleSendableEmail::new(
        "user@localhost",
        vec!["root@localhost"],
        "sendmail_id",
        "Hello sendmail",
    );

    let result = sender.send(email);
    println!("{:?}", result);
    assert!(result.is_ok());
}
