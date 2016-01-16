//! Lettre is a mailer written in Rust. It provides a simple email builder and several transports.
//!
//! ## Architecture
//!
//! This mailer is divided into:
//!
//! * An `email` part: builds the email message
//! * A `transport` part: contains the available transports for your emails. To be sendable, the
//!   emails have to implement `SendableEmail`.
//!
//! ## SMTP transport
//!
//! This SMTP follows [RFC
//! 5321](https://tools.ietf.org/html/rfc5321), but is still
//! a work in progress. It is designed to efficiently send emails from an
//! application to a
//! relay email server, as it relies as much as possible on the relay server
//! for sanity and RFC
//! compliance checks.
//!
//! It implements the following extensions:
//!
//! * 8BITMIME ([RFC 6152](https://tools.ietf.org/html/rfc6152))
//! * AUTH ([RFC 4954](http://tools.ietf.org/html/rfc4954)) with PLAIN and
//! CRAM-MD5 mechanisms
//! * STARTTLS ([RFC 2487](http://tools.ietf.org/html/rfc2487))
//! * SMTPUTF8 ([RFC 6531](http://tools.ietf.org/html/rfc6531))
//!
//! ### Simple example
//!
//! This is the most basic example of usage:
//!
//! ```rust
//! use lettre::transport::smtp::{SmtpTransport, SmtpTransportBuilder};
//! use lettre::email::EmailBuilder;
//! use lettre::transport::EmailTransport;
//!
//! // Create an email
//! let email = EmailBuilder::new()
//!     // Addresses can be specified by the couple (email, alias)
//!     .to(("user@example.org", "Firstname Lastname"))
//!     // ... or by an address only
//!     .from("user@example.com")
//!     .subject("Hi, Hello world")
//!     .body("Hello world.")
//!     .build().unwrap();
//!
//! // Open a local connection on port 25
//! let mut mailer =
//! SmtpTransportBuilder::localhost().unwrap().build();
//! // Send the email
//! let result = mailer.send(email);
//!
//! assert!(result.is_ok());
//! ```
//!
//! ### Complete example
//!
//! ```rust,no_run
//! use lettre::email::EmailBuilder;
//! use lettre::transport::smtp::{SecurityLevel, SmtpTransport,
//! SmtpTransportBuilder};
//! use lettre::transport::smtp::authentication::Mechanism;
//! use lettre::transport::smtp::SUBMISSION_PORT;
//! use lettre::transport::EmailTransport;
//!
//! let mut builder = EmailBuilder::new();
//! builder = builder.to(("user@example.org", "Alias name"));
//! builder = builder.cc(("user@example.net", "Alias name"));
//! builder = builder.from("no-reply@example.com");
//! builder = builder.from("no-reply@example.eu");
//! builder = builder.sender("no-reply@example.com");
//! builder = builder.subject("Hello world");
//! builder = builder.body("Hi, Hello world.");
//! builder = builder.reply_to("contact@example.com");
//! builder = builder.add_header(("X-Custom-Header", "my header"));
//!
//! let email = builder.build().unwrap();
//!
//! // Connect to a remote server on a custom port
//! let mut mailer = SmtpTransportBuilder::new(("server.tld",
//! SUBMISSION_PORT)).unwrap()
//!     // Set the name sent during EHLO/HELO, default is `localhost`
//!     .hello_name("my.hostname.tld")
//!     // Add credentials for authentication
//!     .credentials("username", "password")
//!     // Specify a TLS security level. You can also specify an SslContext with
//!     // .ssl_context(SslContext::Ssl23)
//!     .security_level(SecurityLevel::AlwaysEncrypt)
//!     // Enable SMTPUTF8 is the server supports it
//!     .smtp_utf8(true)
//!     // Configure accepted authetication mechanisms
//!     .authentication_mechanisms(vec![Mechanism::CramMd5])
//!     // Enable connection reuse
//!     .connection_reuse(true).build();
//!
//! let result_1 = mailer.send(email.clone());
//! assert!(result_1.is_ok());
//!
//! // The second email will use the same connection
//! let result_2 = mailer.send(email);
//! assert!(result_2.is_ok());
//!
//! // Explicitely close the SMTP transaction as we enabled connection reuse
//! mailer.close();
//! ```
//!
//! ### Lower level
//!
//! You can also send commands, here is a simple email transaction without
//! error handling:
//!
//! ```rust
//! use lettre::transport::smtp::SMTP_PORT;
//! use lettre::transport::smtp::client::Client;
//! use lettre::transport::smtp::client::net::NetworkStream;
//!
//! let mut email_client: Client<NetworkStream> = Client::new();
//! let _ = email_client.connect(&("localhost", SMTP_PORT), None);
//! let _ = email_client.ehlo("my_hostname");
//! let _ = email_client.mail("user@example.com", None);
//! let _ = email_client.rcpt("user@example.org");
//! let _ = email_client.data();
//! let _ = email_client.message("Test email");
//! let _ = email_client.quit();
//! ```
//!
//! ## Stub transport
//!
//! The stub transport only logs message envelope and drops the content. It can be useful for
//! testing purposes.
//!
//! ```rust
//! use lettre::transport::stub::StubEmailTransport;
//! use lettre::transport::EmailTransport;
//! use lettre::email::EmailBuilder;
//!
//! let mut sender = StubEmailTransport;
//! let email = EmailBuilder::new()
//!                     .to("root@localhost")
//!                     .from("user@localhost")
//!                     .body("Hello World!")
//!                     .subject("Hello")
//!                     .build()
//!                     .unwrap();
//! let result = sender.send(email);
//! assert!(result.is_ok());
//! ```
//!
//! Will log the line:
//!
//! ```text
//! b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
//! ```
//!
//! ## File transport
//!
//! The file transport writes the emails to the given directory. The name of the file will be
//! `message_id.txt`.
//! It can be useful for testing purposes.
//!
//! ```rust
//! use std::env::temp_dir;
//!
//! use lettre::transport::file::FileEmailTransport;
//! use lettre::transport::EmailTransport;
//! use lettre::email::{EmailBuilder, SendableEmail};
//!
//! // Write to the local temp directory
//! let mut sender = FileEmailTransport::new(temp_dir());
//! let email = EmailBuilder::new()
//!                 .to("root@localhost")
//!                 .from("user@localhost")
//!                 .body("Hello World!")
//!                 .subject("Hello")
//!                 .build()
//!                 .unwrap();
//!
//! let result = sender.send(email);
//! assert!(result.is_ok());
//! ```
//! Example result in `/tmp/b7c211bc-9811-45ce-8cd9-68eab575d695.txt`:
//!
//! ```text
//! b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
//! To: <root@localhost>
//! From: <user@localhost>
//! Subject: Hello
//! Date: Sat, 31 Oct 2015 13:42:19 +0100
//! Message-ID: <b7c211bc-9811-45ce-8cd9-68eab575d695.lettre@localhost>
//!
//! Hello World!
//! ```


#![deny(missing_docs, unsafe_code, unstable_features)]

#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate crypto;
extern crate time;
extern crate uuid;
extern crate email as email_format;
extern crate bufstream;
extern crate openssl;

pub mod transport;
pub mod email;
