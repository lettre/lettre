extern crate lettre;

use lettre::{EmailAddress, EmailTransport, SimpleSendableEmail};
use lettre::stub::StubEmailTransport;

#[test]
fn stub_transport() {
    let mut sender_ok = StubEmailTransport::new_positive();
    let mut sender_ko = StubEmailTransport::new(Err(()));

    let email = SimpleSendableEmail::new(EmailAddress::new("user@localhost".to_string()),
                                         vec![EmailAddress::new("root@localhost".to_string())],
                                         "stub_id".to_string(),
                                         "Hello stub".to_string());

    sender_ok.send(&email).unwrap();
    sender_ko.send(&email).unwrap_err();
}
