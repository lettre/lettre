use std::fs;

use lettre::{
    message::header::ContentType,
    transport::smtp::{
        authentication::Credentials,
        client::{tls, Tls},
    },
    Message, SmtpTransport, Transport,
};

fn main() {
    tracing_subscriber::fmt::init();

    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();

    // Use a custom certificate stored on disk to securely verify the server's certificate
    let pem_cert = fs::read("certificate.pem").unwrap();
    let cert = tls::native_tls::Certificate::from_pem(&pem_cert).unwrap();
    let tls = tls::TlsParametersBuilder::<tls::NativeTls>::new("smtp.server.com".to_owned())
        .add_root_certificate(cert)
        .build_legacy()
        .unwrap();

    let creds = Credentials::new("smtp_username".to_owned(), "smtp_password".to_owned());

    // Open a remote connection to the smtp server
    let mailer = SmtpTransport::builder_dangerous("smtp.server.com")
        .port(465)
        .tls(Tls::Wrapper(tls))
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}
