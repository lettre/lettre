use lettre::{AsyncSmtpTransport, AsyncStd1Executor, AsyncTransport, Message};
use rsasl::prelude::SASLConfig;

#[async_std::main]
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
    let mailer: AsyncSmtpTransport<AsyncStd1Executor> =
        AsyncSmtpTransport::<AsyncStd1Executor>::starttls_relay("smtp.gmail.com")
            .unwrap()
            .sasl_config(config)
            .build();

    // Send the email
    match mailer.send(email).await {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}
