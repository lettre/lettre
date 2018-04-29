extern crate env_logger;
extern crate lettre;

use lettre::{EmailAddress, Envelope, SendableEmail, SmtpClient, Transport};

fn main() {
    env_logger::init();

    let email = SendableEmail::new(
        Envelope::new(
            Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
            vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
        ).unwrap(),
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
