// This line is only to make it compile from lettre's examples folder,
// since it uses Rust 2018 crate renaming to import tokio.
// Won't be needed in user's code.
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use rsasl::prelude::SASLConfig;
use tokio1_crate as tokio;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new async year")
        .body(String::from("Be happy with async!"))
        .unwrap();

    let config = SASLConfig::with_credentials(
        None,
        "smtp_username".to_string(),
        "smtp_password".to_string(),
    )
    .unwrap();

    // Open a remote connection to gmail using STARTTLS
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp.gmail.com")
            .unwrap()
            .sasl_config(config)
            .build();

    // Send the email
    match mailer.send(email).await {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}
