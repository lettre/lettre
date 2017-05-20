extern crate lettre;

use lettre::{EmailTransport, SimpleSendableEmail};
use lettre::stub::StubEmailTransport;

#[test]
fn stub_transport() {
    let mut sender = StubEmailTransport;
    let email = SimpleSendableEmail::new("user@localhost",
                                         vec!["root@localhost"],
                                         "stub_id",
                                         "Hello stub");

    let result = sender.send(email);
    assert!(result.is_ok());
}
