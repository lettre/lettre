extern crate env_logger;
extern crate lettre;

use lettre::{Message, SmtpClient, Transport};

fn main() {
    env_logger::init();
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body("Be happy!")
        .unwrap();

    // Open a local connection on port 25
    let mut mailer = SmtpClient::new_unencrypted_localhost().unwrap().transport();
    // Send the email
    let result = mailer.send(&email);

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
}
