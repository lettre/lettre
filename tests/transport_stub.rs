use lettre::stub::StubTransport;
use lettre::{Address, Email, Envelope, Transport};
use std::str::FromStr;

#[test]
fn stub_transport() {
    let mut sender_ok = StubTransport::new_positive();
    let mut sender_ko = StubTransport::new(Err(()));
    let email_ok = Email::new(
        Envelope::new(
            Some(Address::from_str("user@localhost").unwrap()),
            vec![Address::from_str("root@localhost").unwrap()],
        )
        .unwrap(),
        "id".to_string(),
        "Hello ß☺ example".to_string().into_bytes(),
    );
    let email_ko = Email::new(
        Envelope::new(
            Some(Address::from_str("user@localhost").unwrap()),
            vec![Address::from_str("root@localhost").unwrap()],
        )
        .unwrap(),
        "id".to_string(),
        "Hello ß☺ example".to_string().into_bytes(),
    );

    sender_ok.send(email_ok).unwrap();
    sender_ko.send(email_ko).unwrap_err();
}
