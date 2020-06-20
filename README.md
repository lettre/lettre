<h1 align="center">lettre</h1>
<div align="center">
 <strong>
   A mailer library for Rust
 </strong>
</div>

<br />

<div align="center">
  <a href="https://docs.rs/lettre">
    <img src="https://docs.rs/lettre/badge.svg"
      alt="docs" />
  </a>
  <a href="https://crates.io/crates/lettre">
    <img src="https://img.shields.io/crates/d/lettre.svg"
      alt="downloads" />
  </a>
  <br />
  <a href="https://gitter.im/lettre/lettre">
    <img src="https://badges.gitter.im/lettre/lettre.svg"
      alt="chat on gitter" />
  </a>
  <a href="https://lettre.at">
    <img src="https://img.shields.io/badge/visit-website-blueviolet"
      alt="website" />
  </a>
</div>

---

## Features

Lettre provides the following features:

* Multiple transport methods
* Unicode support (for email content and addresses)
* Secure delivery with SMTP using encryption and authentication
* Easy email builders

Lettre does not provide (for now):

* Async support
* Email parsing

## Examples

This library requires Rust 1.20 or newer.
To use this library, add the following to your `Cargo.toml`:


```toml
[dependencies]
lettre = "0.9"
lettre_email = "0.9"
```

```rust,no_run
use lettre::{EmailTransport, SmtpTransport};
use lettre_email::EmailBuilder;
use std::path::Path;

let email = EmailBuilder::new()
    // Addresses can be specified by the tuple (email, alias)
    .to(("user@example.org", "Firstname Lastname"))
    // ... or by an address only
    .from("user@example.com")
    .subject("Hi, Hello world")
    .text("Hello world.")
    .build()
    .unwrap();

// Open a local connection on port 25
let mut mailer = SmtpTransport::builder_unencrypted_localhost().unwrap()
                                                                   .build();
// Send the email
let result = mailer.send(&email);

if result.is_ok() {
    println!("Email sent");
} else {
    println!("Could not send email: {:?}", result);
}

assert!(result.is_ok());
```

### Sending HTML with UTF-8 on gmail(emojis, Chinese, etc)


```rust,norun
use lettre::{
    message::{header, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

    match Message::builder()
        .header(header::ContentType(
            "text/html; charset=utf8".parse().unwrap(),
        ))
        .from(sender.parse().unwrap())
        .to(receiver.parse().unwrap())
        .subject("Validator Signing Report")
        .singlepart(
            SinglePart::builder()
                .header(header::ContentType(
                    "text/plain; charset=utf8".parse().unwrap(),
                ))
                .header(header::ContentTransferEncoding::Binary)
	// assume that report is an string of the HTML document with utf8 things in it
                .body(report),
        ) {
        Err(reason) => return eprintln!("issue {:?}", reason),
        Ok(email) => {
            let mailer = SmtpTransport::relay("smtp.gmail.com")
                .unwrap()
                .credentials(creds)
                .build();
            match mailer.send(&email) {
                Ok(b) => println!("everything sent well  {:?}", b),
                Err(reason) => eprintln!("issue sending out email {}", reason),
            }
        }
    }
```

## Testing

The `lettre` tests require an open mail server listening locally on port 2525 and the `sendmail` command.

## Code of conduct

Anyone who interacts with Lettre in any space, including but not limited to
this GitHub repository, must follow our [code of conduct](https://github.com/lettre/lettre/blob/master/CODE_OF_CONDUCT.md).

## License

This program is distributed under the terms of the MIT license.

The builder comes from [emailmessage-rs](https://github.com/katyo/emailmessage-rs) by
Kayo, under MIT license.

See [LICENSE](./LICENSE) for details.
