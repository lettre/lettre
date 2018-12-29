extern crate lettre_email;
extern crate lettre;
use lettre_email::EmailBuilder;
use lettre::{EmailAddress, Envelope};

#[test]
fn build_with_envelope_test() {
    let e = Envelope::new(
        Some(EmailAddress::new("from@example.org".to_string()).unwrap()),
        vec![EmailAddress::new("to@example.org".to_string()).unwrap()],
    ).unwrap();
    let _email = EmailBuilder::new()
        .envelope(e)
        .subject("subject")
        .text("message")
        .build()
        .unwrap();
}