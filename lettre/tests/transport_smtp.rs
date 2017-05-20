extern crate lettre;

use lettre::{EmailTransport, SimpleSendableEmail};
use lettre::smtp::SecurityLevel;
use lettre::smtp::SmtpTransportBuilder;

#[test]
fn smtp_transport_simple() {
    let mut sender = SmtpTransportBuilder::new("127.0.0.1:2525")
        .unwrap()
        .security_level(SecurityLevel::Opportunistic)
        .build();
    let email = SimpleSendableEmail::new("user@localhost",
                                         vec!["root@localhost"],
                                         "smtp_id",
                                         "Hello smtp");

    let result = sender.send(email);
    assert!(result.is_ok());
}
