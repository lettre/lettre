use lettre::stub::StubTransport;
use lettre::{EmailAddress, Envelope, SendableEmail, Transport};

#[test]
fn stub_transport() {
    let mut sender_ok = StubTransport::new_positive();
    let mut sender_ko = StubTransport::new(Err(()));
    let email_ok = SendableEmail::new(
        Envelope::new(
            Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
            vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
        )
        .unwrap(),
        "id".to_string(),
        "Hello ß☺ example".to_string().into_bytes(),
    );
    let email_ko = SendableEmail::new(
        Envelope::new(
            Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
            vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
        )
        .unwrap(),
        "id".to_string(),
        "Hello ß☺ example".to_string().into_bytes(),
    );

    sender_ok.send(email_ok).unwrap();
    sender_ko.send(email_ko).unwrap_err();
}
