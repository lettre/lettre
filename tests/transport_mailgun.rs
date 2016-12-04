extern crate lettre;

use lettre::email::EmailBuilder;
use lettre::transport::EmailTransport;
use lettre::transport::mailgun::MailgunTransport;
use lettre::transport::mailgun::error::Error;

#[test]
fn mailgun_transport_simple() {
    use std::env::var;
    let domain = var("MAILGUN_DOMAIN").expect("Need MAILGUN_DOMAIN env variable");
    let api_key = var("MAILGUN_APIKEY").expect("Need MAILGUN_APIKEY env variable");
    let mut sender = MailgunTransport::new(domain, api_key);
    let email = EmailBuilder::new()
        .to("neikos@neikos.email")
        .from("hello@neikos.email")
        .body("Hello World!")
        .subject("Hello sendmail")
        .build()
        .unwrap();
    let mut result = sender.send(email);
    println!("{:#?}", result);
    if let Some(&mut Error::Mailgun(ref mut mg)) = result.as_mut().err() {
        use std::io::Read;

        let mut buf = Vec::new();
        while let Ok(n) = mg.read(&mut buf) {
            // Wait
            if n == 0 { break }
        }
        println!("{:#?}", buf);
        panic!();
    }
    assert!(result.is_ok());
}
