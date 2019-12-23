extern crate env_logger;
extern crate lettre;

use lettre::{Address, Email, Envelope, SmtpClient, Transport};
use std::str::FromStr;

fn main() {
    env_logger::init();

    let email = Email::new(
        Envelope::new(
            Some(Address::from_str("user@localhost").unwrap()),
            vec![Address::from_str("root@localhost").unwrap()],
        )
        .unwrap(),
        "id".to_string(),
        "Hello ß☺ example".to_string().into_bytes(),
    );

    // Open a local connection on port 25
    let mut mailer = SmtpClient::new_unencrypted_localhost().unwrap().transport();
    // Send the email
    let result = mailer.send(email);

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
