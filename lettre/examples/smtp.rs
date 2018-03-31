extern crate env_logger;
extern crate lettre;

use lettre::{EmailTransport, SimpleSendableEmail, SmtpTransport};

fn main() {
    env_logger::init();

    let email = SimpleSendableEmail::new(
        "user@localhost".to_string(),
        &["root@localhost".to_string()],
        "my-message-id".to_string(),
        "Hello ß☺ example".to_string(),
    ).unwrap();

    // Open a local connection on port 25
    let mut mailer = SmtpTransport::builder_unencrypted_localhost()
        .unwrap()
        .build();
    // Send the email
    let result = mailer.send(&email);

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
