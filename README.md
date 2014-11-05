rust-smtp [![Build Status](https://travis-ci.org/amousset/rust-smtp.svg?branch=master)](https://travis-ci.org/amousset/rust-smtp)
=========

This library implements an SMTP library and a simple client.

Rust versions
-------------

This library is designed for Rust 0.13.0-nightly (master).

Install
-------

If you're using the library in a program, just add this to your `Cargo.toml`:

```toml
[dependencies.smtp]
git = "https://github.com/amousset/rust-smtp.git"
```

Otherwise, clone this repo and run `cargo build`.

Example
-------

There is an example command-line program:
```
$ cargo test
$ env RUST_LOG=info ./target/examples/client -r sender@localhost recipient@localhost < email.txt
INFO:smtp::client: Connection established to localhost[127.0.0.1]:25
INFO:smtp::client: from=<sender@localhost>, size=989, nrcpt=1
INFO:smtp::client: to=<recipient@localhost>, status=sent (250 2.0.0 Ok: queued as 9D28F1C0A51)
```

Run `./target/examples/client -h` to get a list of available options.

Documentation
-------------

You can build the documentation with `cargo doc`. It is also available on [GitHub pages](http://amousset.github.io/rust-smtp/smtp/).

License
-------

This program is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE, LICENSE-MIT, and COPYRIGHT for details.
