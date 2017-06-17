extern crate lettre;

use lettre::{EmailTransport, SimpleSendableEmail};
use lettre::smtp::{SecurityLevel, SmtpTransportBuilder};

fn main() {
    let email = SimpleSendableEmail::new(
        "user@localhost",
        vec!["root@localhost"],
        "file_id",
        "Hello ß☺ example",
    );

    // Open a local connection on port 25
    let mut mailer = SmtpTransportBuilder::localhost().unwrap().security_level(SecurityLevel::Opportunistic).build();
    // Send the email
    let result = mailer.send(email);

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
