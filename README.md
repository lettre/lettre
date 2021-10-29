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
  <a href="https://lettre.rs">
    <img src="https://img.shields.io/badge/visit-website-blueviolet"
      alt="website" />
  </a>
</div>

<div align="center">
  <a href="https://deps.rs/crate/lettre/0.10.0-rc.3">
    <img src="https://deps.rs/crate/lettre/0.10.0-rc.3/status.svg"
      alt="dependency status" />
  </a>
</div>

---

**NOTE**: this readme refers to the 0.10 version of lettre, which is
in release candidate state. Use the [`v0.9.x`](https://github.com/lettre/lettre/tree/v0.9.x)
branch for the previous stable release.

0.10 is already widely used and is already thought to be more reliable than 0.9, so it should generally be used
for new projects.

We'd love to hear your feedback about 0.10 design and APIs before final release!
Start a [discussion](https://github.com/lettre/lettre/discussions) in the repository, whether for
feedback or if you need help or advice using or upgrading lettre 0.10.

---

## Features

Lettre provides the following features:

* Multiple transport methods
* Unicode support (for email content and addresses)
* Secure delivery with SMTP using encryption and authentication
* Easy email builders
* Async support

Lettre does not provide (for now):

* Email parsing

## Example

This library requires Rust 1.52.1 or newer.
To use this library, add the following to your `Cargo.toml`:


```toml
[dependencies]
lettre = "0.10.0-rc.3"
```

```rust,no_run
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

let email = Message::builder()
    .from("NoBody <nobody@domain.tld>".parse().unwrap())
    .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
    .to("Hei <hei@domain.tld>".parse().unwrap())
    .subject("Happy new year")
    .body(String::from("Be happy!"))
    .unwrap();

let creds = Credentials::new("smtp_username".to_string(), "smtp_password".to_string());

// Open a remote connection to gmail
let mailer = SmtpTransport::relay("smtp.gmail.com")
    .unwrap()
    .credentials(creds)
    .build();

// Send the email
match mailer.send(&email) {
    Ok(_) => println!("Email sent successfully!"),
    Err(e) => panic!("Could not send email: {:?}", e),
}
```

## Testing

The `lettre` tests require an open mail server listening locally on port 2525 and the `sendmail` command. If you have python installed 
such a server can be launched with `python -m smtpd -n -c DebuggingServer localhost:2525`

Alternatively only unit tests can be run by doing `cargo test --lib`.

## Troubleshooting

These are general steps to be followed when troubleshooting SMTP related issues.
- Ensure basic connectivity, ensure requisite ports are open and daemons are listening.
- Confirm that your service provider allows traffic on the ports being used for mail transfer. (traffic on ports 25, 2525, 587, and 465 is commonly blocked by some service providers.)
- Check if your service provider requires using an SMTP relay, and ensure you are correctly authenticated to use this relay.
- Validate your DNS records. (DMARC, SPF, DKIM, MX)
- Verify your SSL/TLS certificates are setup properly. (ensure cert and bundled ca-certificates are up to date and valid, check file permissions)
- Determine if SSL and/or StarTLS are supported and setup as required by remote hosts.
- Investigate if filtering, formatting, or filesize limits are causing messages to be lost, delayed, or blocked by relays or remote hosts.

## Code of conduct

Anyone who interacts with Lettre in any space, including but not limited to
this GitHub repository, must follow our [code of conduct](https://github.com/lettre/lettre/blob/master/CODE_OF_CONDUCT.md).

## License

This program is distributed under the terms of the MIT license.

The builder comes from [emailmessage-rs](https://github.com/katyo/emailmessage-rs) by
Kayo, under MIT license.

See [LICENSE](./LICENSE) for details.
