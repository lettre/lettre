// This line is only to make it compile from lettre's examples folder,
// since it uses Rust 2018 crate renaming to import tokio.
// Won't be needed in user's code.
use tokio02_crate as tokio;

use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, Message, Tokio02Connector,
    Tokio02Transport,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new async year")
        .body("Be happy with async!")
        .unwrap();

    let creds = Credentials::new("smtp_username".to_string(), "smtp_password".to_string());

    // Open a remote connection to gmail using STARTTLS
    let mailer = AsyncSmtpTransport::<Tokio02Connector>::starttls_relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(email).await {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
