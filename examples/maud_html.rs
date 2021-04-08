use lettre::{
    message::{header, MultiPart, SinglePart},
    FileTransport, Message, Transport,
};
use maud::html;

fn main() {
    // The recipient's name. We might obtain this from a form or their email address.
    let recipient = "Hei";

    // Create the html we want to send.
    let html = html! {
        head {
            title { "Hello from Lettre!" }
            style type="text/css" {
                "h2, h4 { font-family: Arial, Helvetica, sans-serif; }"
            }
        }
        div style="display: flex; flex-direction: column; align-items: center;" {
            h2 { "Hello from Lettre!" }
            // Substitute in the name of our recipient.
            p { "Dear " (recipient) "," }
            p { "This email was sent with Lettre, a mailer library for Rust!"}
            p {
                "This example uses "
                a href="https://crates.io/crates/maud" { "maud" }
                ". It is about 20% cooler than the basic HTML example."
            }
        }
    };

    // Build the message.
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Hello from Lettre!")
        .multipart(
            MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(String::from("Hello from Lettre! A mailer library for Rust")), // Every message should have a plain text fallback.
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html.into_string()),
                ),
        )
        .expect("failed to build email");

    // Create our mailer. Please see the other examples for creating SMTP mailers.
    // The path given here must exist on the filesystem.
    let mailer = FileTransport::new("./");

    // Store the message when you're ready.
    mailer.send(&email).expect("failed to deliver message");
}
