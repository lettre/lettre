# Lettre Examples

This folder contains examples showing how to use lettre in your own projects.

## Message builder examples

- [basic_html.rs] - Create an HTML email.
- [maud_html.rs] - Create an HTML email using a [maud](https://github.com/lambda-fairy/maud) template.

## SMTP Examples

- [smtp.rs] - Send an email using a local SMTP daemon on port 25 as a relay.
- [smtp_tls.rs] - Send an email over SMTP encrypted with TLS and authenticating with username and password.
- [smtp_starttls.rs] - Send an email over SMTP with STARTTLS and authenticating with username and password.
- [smtp_selfsigned.rs] - Send an email over SMTP encrypted with TLS using a self-signed certificate and authenticating with username and password.
- The [smtp_tls.rs] and [smtp_starttls.rs] examples also feature `async`hronous implementations powered by [Tokio](https://tokio.rs/).
  These files are prefixed with `tokio02_`, `tokio1_` or `asyncstd1_`.

[basic_html.rs]: ./basic_html.rs
[maud_html.rs]: ./maud_html.rs
[smtp.rs]: ./smtp.rs
[smtp_tls.rs]: ./smtp_tls.rs
[smtp_starttls.rs]: ./smtp_starttls.rs
[smtp_selfsigned.rs]: ./smtp_selfsigned.rs
