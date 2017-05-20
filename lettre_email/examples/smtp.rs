extern crate lettre;
extern crate lettre_email;

use lettre_email::email::EmailBuilder;
use lettre::EmailTransport;
use lettre::smtp::SmtpTransportBuilder;

fn main() {
    let email = EmailBuilder::new()
        // Addresses can be specified by the tuple (email, alias)
        .to(("user@example.org", "Firstname Lastname"))
        // ... or by an address only
        .from("user@example.com")
        .subject("Hi, Hello world")
        .text("Hello world.")
        .build()
        .unwrap();

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
