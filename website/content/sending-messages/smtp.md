+++
date = "2017-05-21T23:46:17+02:00"
title = "SMTP transport"
toc = true
weight = 2

+++

This transport uses the SMTP protocol to send emails over the network (locally or remotely).

It is desinged to be:

* Secured: email are encrypted by default
* Modern: Unicode support for email content and sender/recipient adresses when compatible
* Fast: supports tcp connection reuse

This client is designed to send emails to a relay server, and should *not* be used to send
emails directly to the destination.

The relay server can be the local email server, a specific host or a third-party service.

#### Simple example

This is the most basic example of usage:

{{< highlight rust >}}
use lettre::{SimpleSendableEmail, EmailTransport};
use lettre::smtp::SmtpTransportBuilder;
use lettre::smtp::SecurityLevel;

let email = SimpleSendableEmail::new(
                "user@localhost",
                vec!["root@localhost"],
                "message_id",
                "Hello world"
            );

// Open a local connection on port 25
let mut mailer =
SmtpTransportBuilder::localhost().unwrap().security_level(SecurityLevel::Opportunistic).build();
// Send the email
let result = mailer.send(email);

assert!(result.is_ok());
{{< /highlight >}}

#### Complete example

{{< highlight rust >}}
use lettre::smtp::{SecurityLevel, SmtpTransport,
SmtpTransportBuilder};
use lettre::smtp::authentication::Mechanism;
use lettre::smtp::SUBMISSION_PORT;
use lettre::{SimpleSendableEmail, EmailTransport};

let email = SimpleSendableEmail::new(
                "user@localhost",
                vec!["root@localhost"],
                "message_id",
                "Hello world"
            );

// Connect to a remote server on a custom port
let mut mailer = SmtpTransportBuilder::new(("server.tld",
SUBMISSION_PORT)).unwrap()
    // Set the name sent during EHLO/HELO, default is `localhost`
    .hello_name("my.hostname.tld")
    // Add credentials for authentication
    .credentials("username", "password")
    // Specify a TLS security level. You can also specify an SslContext with
    // .ssl_context(SslContext::Ssl23)
    .security_level(SecurityLevel::AlwaysEncrypt)
    // Enable SMTPUTF8 if the server supports it
    .smtp_utf8(true)
    // Configure expected authentication mechanism
    .authentication_mechanism(Mechanism::CramMd5)
    // Enable connection reuse
    .connection_reuse(true).build();

let result_1 = mailer.send(email.clone());
assert!(result_1.is_ok());

// The second email will use the same connection
let result_2 = mailer.send(email);
assert!(result_2.is_ok());

// Explicitly close the SMTP transaction as we enabled connection reuse
mailer.close();
{{< /highlight >}}

#### Lower level

You can also send commands, here is a simple email transaction without
error handling:

{{< highlight rust >}}
use lettre::smtp::SMTP_PORT;
use lettre::smtp::client::Client;
use lettre::smtp::client::net::NetworkStream;

let mut email_client: Client<NetworkStream> = Client::new();
let _ = email_client.connect(&("localhost", SMTP_PORT), None);
let _ = email_client.ehlo("my_hostname");
let _ = email_client.mail("user@example.com", None);
let _ = email_client.rcpt("user@example.org");
let _ = email_client.data();
let _ = email_client.message("Test email");
let _ = email_client.quit();
{{< /highlight >}}

