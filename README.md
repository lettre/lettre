# lettre

[![Build Status](https://travis-ci.org/lettre/lettre.svg?branch=master)](https://travis-ci.org/lettre/lettre)
[![Build status](https://ci.appveyor.com/api/projects/status/mpwglemugjtkps2d/branch/master?svg=true)](https://ci.appveyor.com/project/amousset/lettre/branch/master)
[![Crate](https://img.shields.io/crates/v/lettre.svg)](https://crates.io/crates/lettre)
[![Docs](https://docs.rs/lettre/badge.svg)](https://docs.rs/lettre/)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Gitter](https://badges.gitter.im/lettre/lettre.svg)](https://gitter.im/lettre/lettre?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

This is an email library written in Rust.

## Features

Lettre provides the following features:

* Multiple transport methods
* Unicode support (for email content and addresses)
* Secure delivery with SMTP using encryption and authentication
* Easy email builders

## Example

```rust,no_run
extern crate lettre;
extern crate lettre_email;
extern crate mime;

use lettre::{EmailTransport, SmtpTransport};
use lettre_email::EmailBuilder;
use std::path::Path;

fn main() {
    let email = EmailBuilder::new()
        // Addresses can be specified by the tuple (email, alias)
        .to(("user@example.org", "Firstname Lastname"))
        // ... or by an address only
        .from("user@example.com")
        .subject("Hi, Hello world")
        .text("Hello world.")
        .attachment(Path::new("Cargo.toml"), None, &mime::TEXT_PLAIN).unwrap()
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
}
```

## Documentation

Released versions:

* [latest](https://docs.rs/lettre/)
* [v0.8.0](https://docs.rs/lettre/0.8.0/lettre/)
* [v0.7.0](https://docs.rs/lettre/0.7.0/lettre/)
* [v0.6.2](https://docs.rs/lettre/0.6.2/lettre/)
* [v0.6.1](https://docs.rs/lettre/0.6.1/lettre/)
* [v0.6.0](https://docs.rs/lettre/0.6.0/lettre/)
* [v0.5.1](https://docs.rs/lettre/0.5.1/lettre/)

## Install

This library requires rust 1.18 or newer.
To use this library, add the following to your `Cargo.toml`:

```toml
[dependencies]
lettre = "0.8"
lettre_email = "0.8"
```

## Testing

The `lettre` tests require an open mail server listening locally on port 2525 and the `sendmail` command.

## Code of conduct

Anyone who interacts with Lettre in any space, including but not limited to
this GitHub repository, must follow our [code of conduct](https://github.com/lettre/lettre/blob/master/CODE_OF_CONDUCT.md).

## License

This program is distributed under the terms of the MIT license.

See [LICENSE](./LICENSE) for details.
