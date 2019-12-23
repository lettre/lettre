extern crate lettre;

use lettre::smtp::authentication::Credentials;
use lettre::{Address, Email, Envelope, SmtpClient, Transport};
use std::str::FromStr;

fn main() {
    let email = Email::new(
        Envelope::new(
            Some(Address::from_str("user@gmail.com").unwrap()),
            vec![Address::from_str("root@example.com").unwrap()],
        )
        .unwrap(),
        "id".to_string(),
        "Hello example".to_string().into_bytes(),
    );

    let creds = Credentials::new(
        "example_username".to_string(),
        "example_password".to_string(),
    );

    // Open a remote connection to gmail
    let mut mailer = SmtpClient::new_simple("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .transport();

    // Send the email
    let result = mailer.send(email);

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
