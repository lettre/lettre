extern crate lettre;

use lettre::{EmailTransport, SimpleSendableEmail};
use lettre::stub::StubEmailTransport;

#[test]
fn stub_transport() {
    let mut sender_ok = StubEmailTransport::new_positive();
    let mut sender_ko = StubEmailTransport::new(Err(()));
    let email = SimpleSendableEmail::new(
        "user@localhost".to_string(),
        &["root@localhost".to_string()],
        "stub_id".to_string(),
        "Hello stub".to_string(),
    ).unwrap();

    sender_ok.send(&email).unwrap();
    sender_ko.send(&email).unwrap_err();
}
