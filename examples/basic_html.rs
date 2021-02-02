use lettre::{
    message::{header, MultiPart, SinglePart},
    FileTransport, Message, Transport,
};

fn main() {
    // The html we want to send.
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hello from Lettre!</title>
</head>
<body>
    <div style="display: flex; flex-direction: column; align-items: center;">
        <h2 style="font-family: Arial, Helvetica, sans-serif;">Hello from Lettre!</h2>
        <h4 style="font-family: Arial, Helvetica, sans-serif;">A mailer library for Rust</h4>
    </div>
</body>
</html>"#;

    // Build the message.
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Hello from Lettre!")
        .multipart(
            MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType(
                            "text/plain; charset=utf8".parse().unwrap(),
                        ))
                        .body(String::from("Hello from Lettre! A mailer library for Rust")), // Every message should have a plain text fallback.
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType(
                            "text/html; charset=utf8".parse().unwrap(),
                        ))
                        .body(String::from(html)),
                ),
        )
        .expect("failed to build email");

    // Create our mailer. Please see the other examples for creating SMTP mailers.
    // The path given here must exist on the filesystem.
    let mailer = FileTransport::new("./");

    // Store the message when you're ready.
    mailer.send(&email).expect("failed to deliver message");
}
