# Lettre Examples
 This folder contains examples showing how to user lettre in your own projects.

## Examples
- [smtp.rs] - Send an email using a local SMTP daemon on port 25 as a relay.
- [smtp_tls.rs] - Send an email over SMTP encrypted with TLS and authenticating with username and password.
- [smtp_starttls.rs] - Send an email over SMTP with STARTTLS and authenticatign with username and password.
- [smtp_selfsigned.rs] - Send an email over SMTP encrypted with TLS using a self-signed certificate and authenticating with username and password.
- The [smtp_tls.rs] and [smtp_starttls.rs] examples also feature `async`hronus implementations powered by [Tokio](https://tokio.rs/).
  These files are prfixed with `tokio02_` or `tokio03_`.

[smtp.rs]: ./smtp.rs
[smtp_tls.rs]: ./smtp_tls.rs
[smtp_starttls.rs]: ./smtp_starttls.rs
[smtp_selfsigned.rs]: ./smtp_selfsigned.rs
