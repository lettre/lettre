extern crate lettre;
extern crate lettre_email;

use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, Transport};
use lettre_email::Email;

fn main() {
    let email = Email::builder()
        .to("to@example.org")
        .from("from@example.com")
        .subject("subject")
        .text("message")
        .build()
        .unwrap();

    let creds = Credentials::new(
        "example_username".to_string(),
        "example_password".to_string(),
    );

    // Open connection to gmail
    let mut mailer = SmtpClient::new_simple("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .transport();

    // Send the email
    let result = mailer.send(email.into());

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
