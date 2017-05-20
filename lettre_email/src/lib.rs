//! Lettre is a mailer written in Rust. It provides a simple email builder and several transports.
//!
//! ## Overview
//!
//! The `email` part builds email messages. For now, it does not support attachments.
//! An email is built using an `EmailBuilder`. The simplest email could be:
//!
//! ```rust
//! use lettre_email::email::EmailBuilder;
//!
//! // Create an email
//! let email = EmailBuilder::new()
//!     // Addresses can be specified by the tuple (email, alias)
//!     .to(("user@example.org", "Firstname Lastname"))
//!     // ... or by an address only
//!     .from("user@example.com")
//!     .subject("Hi, Hello world")
//!     .text("Hello world.")
//!     .build();
//!
//! assert!(email.is_ok());
//! ```
//!
//! When the `build` method is called, the `EmailBuilder` will add the missing headers (like
//! `Message-ID` or `Date`) and check for missing necessary ones (like `From` or `To`). It will
//! then generate an `Email` that can be sent.
//!
//! The `text()` method will create a plain text email, while the `html()` method will create an
//! HTML email. You can use the `alternative()` method to provide both versions, using plain text
//! as fallback for the HTML version.
//!
//! Below is a more complete example, not using method chaining:
//!
//! ```rust
//! use lettre_email::email::EmailBuilder;
//!
//! let mut builder = EmailBuilder::new();
//! builder.add_to(("user@example.org", "Alias name"));
//! builder.add_cc(("user@example.net", "Alias name"));
//! builder.add_from("no-reply@example.com");
//! builder.add_from("no-reply@example.eu");
//! builder.set_sender("no-reply@example.com");
//! builder.set_subject("Hello world");
//! builder.set_alternative("<h2>Hi, Hello world.</h2>", "Hi, Hello world.");
//! builder.add_reply_to("contact@example.com");
//! builder.add_header(("X-Custom-Header", "my header"));
//!
//! let email = builder.build();
//! assert!(email.is_ok());
//! ```
//!
//! See the `EmailBuilder` documentation for a complete list of methods.
//!

#![deny(missing_docs, unsafe_code, unstable_features, warnings, missing_debug_implementations)]

#[macro_use]
extern crate mime;
extern crate time;
extern crate uuid;
extern crate email as email_format;
extern crate lettre;

pub mod error;
pub mod email;
