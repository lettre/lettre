extern crate lettre;

use lettre::{EmailAddress, EmailTransport, SecurityLevel, SimpleSendableEmail, SmtpTransport};

fn main() {
    let email = SimpleSendableEmail::new(
        EmailAddress::new("user@localhost".to_string()),
        vec![EmailAddress::new("root@localhost".to_string())],
        "file_id".to_string(),
        "Hello ß☺ example".to_string(),
    );

    // Open a local connection on port 25
    let mut mailer = SmtpTransport::builder_localhost()
        .unwrap()
        .security_level(SecurityLevel::Opportunistic)
        .build();
    // Send the email
    let result = mailer.send(email);

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
