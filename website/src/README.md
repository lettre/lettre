# Introduction

Lettre is an email library that allows creating and sending messages. It provides:

* An easy to use email builder
* Pluggable email transports
* Unicode support (for emails and transports, including for sender et recipient addresses when compatible)
* Secure defaults (emails are only sent encrypted by default)

Lettre requires Rust 1.40 or newer. Add the following to your `Cargo.toml`:

```toml
[dependencies]
lettre = "0.10"
```
