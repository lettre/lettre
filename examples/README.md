# Lettre Examples

This folder contains examples showing how to use lettre in your own projects.

## Examples
- [file_html.rs] - Create an HTML email and store it in a local directory using `FileTransport`.
- [smtp.rs] - Send an email using a local SMTP daemon on port 25 as a relay.
- [smtp_tls.rs] - Send an email over SMTP encrypted with TLS and authenticating with username and password.
- [smtp_starttls.rs] - Send an email over SMTP with STARTTLS and authenticating with username and password.
- [smtp_selfsigned.rs] - Send an email over SMTP encrypted with TLS using a self-signed certificate and authenticating with username and password.
- The [smtp_tls.rs] and [smtp_starttls.rs] examples also feature `async`hronous implementations powered by [Tokio](https://tokio.rs/).
  These files are prefixed with `tokio02_` or `tokio03_`.

[file_html.rs]: ./file_html.rs
[smtp.rs]: ./smtp.rs
[smtp_tls.rs]: ./smtp_tls.rs
[smtp_starttls.rs]: ./smtp_starttls.rs
[smtp_selfsigned.rs]: ./smtp_selfsigned.rs
