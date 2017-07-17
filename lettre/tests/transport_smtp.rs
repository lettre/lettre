extern crate lettre;

use lettre::{EmailAddress, EmailTransport, SimpleSendableEmail};
use lettre::smtp::SecurityLevel;
use lettre::smtp::SmtpTransportBuilder;

#[test]
fn smtp_transport_simple() {
    let mut sender = SmtpTransportBuilder::new("127.0.0.1:2525")
        .unwrap()
        .security_level(SecurityLevel::Opportunistic)
        .build();
    let email = SimpleSendableEmail::new(
        EmailAddress::new("user@localhost".to_string()),
        vec![EmailAddress::new("root@localhost".to_string())],
        "smtp_id".to_string(),
        "Hello smtp".to_string(),
    );

    let result = sender.send(email);
    assert!(result.is_ok());
}
