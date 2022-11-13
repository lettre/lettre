use lettre::{Message, SmtpTransport, Transport};

fn main() {
    tracing_subscriber::fmt::init();

    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .to("Idk <idk@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body(String::from("Be happy!"))
        .unwrap();

    // Open a local connection on port 11200
    let mailer = <SmtpTransport<true>>::builder_dangerous("localhost").build();

    // Send the email
    match mailer.send(&email) {
        Ok(responses) => {
            let responses: Vec<_> = responses;
            println!("Email sent successfully: {responses:?}")
        }
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
