extern crate lettre;

use lettre::{EmailTransport, SimpleSendableEmail};
use lettre::smtp::SmtpTransportBuilder;

fn main() {
    let email = SimpleSendableEmail::new(
        "user@localhost",
        vec!["root@localhost"],
        "file_id",
        "Hello file",
    );

    // Open a local connection on port 25
    let mut mailer = SmtpTransportBuilder::localhost().unwrap().build();
    // Send the email
    let result = mailer.send(email);

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
