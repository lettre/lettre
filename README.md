# lettre

**Lettre is a mailer library for Rust.**

![](https://github.com/lettre/lettre/workflows/Continuous%20integration/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/lettre/lettre/badge.svg?branch=master)](https://coveralls.io/github/lettre/lettre?branch=master)

[![Crate](https://img.shields.io/crates/v/lettre.svg)](https://crates.io/crates/lettre)
[![Docs](https://docs.rs/lettre/badge.svg)](https://docs.rs/lettre/)
[![Required Rust version](https://img.shields.io/badge/rustc-1.40-green.svg)]()
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

[![Gitter](https://badges.gitter.im/lettre/lettre.svg)](https://gitter.im/lettre/lettre?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
[![Average time to resolve an issue](https://isitmaintained.com/badge/resolution/lettre/lettre.svg)](https://isitmaintained.com/project/lettre/lettre "Average time to resolve an issue")
[![Percentage of issues still open](https://isitmaintained.com/badge/open/lettre/lettre.svg)](https://isitmaintained.com/project/lettre/lettre "Percentage of issues still open")

Useful links:

* [User documentation](https://lettre.at/)
* [API documentation](https://docs.rs/lettre/)
* [Changelog](https://github.com/lettre/lettre/blob/master/CHANGELOG.md)

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

## Example

This library requires Rust 1.40 or newer.
To use this library, add the following to your `Cargo.toml`:

```toml
[dependencies]
lettre = "0.10"
```

```rust,no_run
extern crate lettre;

use lettre::{SmtpClient, Transport, Message};
use std::convert::TryInto;

fn main() {
    let email = Message::builder()
        // Addresses can be specified by the tuple (email, alias)
        .to(("user@example.org", "Firstname Lastname").try_into().unwrap())
        // ... or by an address only
        .from("user@example.com".parse().unwrap())
        .subject("Hi, Hello world")
        .body("Hello world.")
        //.attachment_from_file(Path::new("Cargo.toml"), None, &TEXT_PLAIN)
        // FIXME add back attachment example
        .build()
        .unwrap();

    // Open a local connection on port 25
    let mut mailer = SmtpClient::new_unencrypted_localhost().unwrap().transport();
    // Send the email
    let result = mailer.send(email);

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    assert!(result.is_ok());
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
