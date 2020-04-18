# Lettre 0.10

*2019.04.12*

## What is `lettre`?

Lettre provides an email client for Rust programs, to allow easily sending emails from Rust
applications with the following focuses

* Ease of use, without particular knowledge about email
* Secure by default
* Modern (support for full internationalization)

Non-goals:

* Implementing email RFCs extensively. The goal is to target a modern and safe subset needed to
  send emails today, with a nice API (i.e. UTF-8 only, etc.). Particularly, lettre
  currently cannot parse emails.

### Background

The `lettre` crate was previously named [`smtp`](https://crates.io/crates/smtp). It was [created](https://github.com/lettre/lettre/commit/270efd193a11e66dce14700a50d3c42c12e725bc) in early 2014 (before cargo, Rust 1.0, etc.).

The first goal was to start a toy project as a pretext to learn Rust. I started with an `smtp` implementation after seeing there was no existing implementation in Rust. Originally, the project aimed at implementing the `SMTP` protocol for client and server.

In 2016, the goal changed, and specialized to email client (as I did not see much use in another SMTP server may it be written in Rust). The project also moved away from "just SMTP" to email client, and was renamed to lettre at this time. Why `lettre`? After some time looking for a fitting name, not already taken by email-related software, I ended up just taking the the French word for "letter"!

## Changes in 0.10

* Replacement of the message implementation (which was based on `rust-email`)
  by a new one based on the `emailmessage` crate. To main goal is to provide
  sane encoding and multipart was a simple implementation (no message parsing).
* Merge of the `lettre_email` crate into `lettre`. This split made not much sense, and the message
  builder is now a feature of the `lettre` crate.
* More features to allow disabling most features.
* Add the option to use `rustls` for TLS.
* Improved parsing of server responses.
* Moved CI from Travis to Github actions.

### Migration from 0.9

TODO

## Road to 1.0

Lettre is now used by several projects, including crates.io itself!
It will be good to have a stable basis for the future.

The plan is that 0.10 is the release preparing the 1.0 in the following months.
I'd also want to add more real-world automated testing with actual mail servers (at least postfix).

`async` is not a goal for 1.0, as it is not as relevant for emails as it is for other ecosystems
(like web), and theadpool-based solutions are in general very suited.

## After

* reuse `smtp` crate for the protocol (a bit like `http`)
* async

If you want to contribute, the `lettre` repo and organizations are definitely open for anything
related to email.
