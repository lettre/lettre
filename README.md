rust-smtp [![Build Status](https://travis-ci.org/amousset/rust-smtp.svg?branch=master)](https://travis-ci.org/amousset/rust-smtp) [![Gitter](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/amousset/rust-smtp?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
=========

This library implements an SMTP library and a simple SMTP client.
See the [documentation](http://amousset.github.io/rust-smtp/smtp/) for more information.

Rust versions
-------------

This library is designed for Rust 1.0.0-nightly (master).

Install
-------

If you're using the library in a program, just add these lines to your `Cargo.toml`:

```toml
[dependencies]
smtp = "*"
```

Otherwise, you can clone this repository and run `cargo build`.

Example
-------

There is an example command-line program included:
```sh
$ cargo test
$ env RUST_LOG=info cargo run --example client -- -s "My subject" -r sender@localhost recipient@localhost < email.txt
INFO:smtp::client: connection established to 127.0.0.1:25
INFO:smtp::client: 1d0467fb21b2454f90a85dd1e0eda839: from=<sender@localhost>
INFO:smtp::client: 1d0467fb21b2454f90a85dd1e0eda839: to=<recipient@localhost>
INFO:smtp::client: 1d0467fb21b2454f90a85dd1e0eda839: conn_use=1, size=1889, status=sent (250 2.0.0 Ok: queued as BAA9C1C0055)
INFO:client: Email sent successfully
```

Run `cargo run --example client -- -h` to get a list of available options.

Tests
-----

You can build and run the tests with `cargo test`. The client does not have tests for now.

Documentation
-------------

You can build the documentation with `cargo doc`. It is also available on [GitHub pages](http://amousset.github.io/rust-smtp/smtp/).

License
-------

This program is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE, LICENSE-MIT, and COPYRIGHT for details.
