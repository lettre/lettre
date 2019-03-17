extern crate lettre;
extern crate lettre_email;
extern crate mime;

use lettre::{SmtpClient, Transport};
use lettre_email::Email;
use std::path::Path;

fn main() {
    let email = Email::builder()
        // Addresses can be specified by the tuple (email, alias)
        .to(("user@example.org", "Firstname Lastname"))
        // ... or by an address only
        .from("user@example.com")
        .subject("Hi, Hello world")
        .text("Hello world.")
        .attachment_from_file(Path::new("Cargo.toml"), None, &mime::TEXT_PLAIN).unwrap()
        .build()
        .unwrap();

    // Open a local connection on port 25
    let mut mailer = SmtpClient::new_unencrypted_localhost().unwrap().transport();
    // Send the email
    let result = mailer.send(email.into());

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
