#### SMTP Transport

This transport uses the SMTP protocol to send emails over the network (locally or remotely).

It is designed to be:

* Secured: email are encrypted by default
* Modern: Unicode support for email content and sender/recipient addresses when compatible
* Fast: supports tcp connection reuse

This client is designed to send emails to a relay server, and should *not* be used to send
emails directly to the destination.

The relay server can be the local email server, a specific host or a third-party service.

#### Simple example

This is the most basic example of usage:

```rust,no_run
extern crate lettre;

use lettre::{SendableEmail, EmailAddress, Transport, Envelope, SmtpClient};

fn main() {
    let email = SendableEmail::new(
        Envelope::new(
            Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
            vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
        ).unwrap(),
        "id".to_string(),
        "Hello world".to_string().into_bytes(),
    );
    
    // Open a local connection on port 25
    let mut mailer =
    SmtpClient::new_unencrypted_localhost().unwrap().transport();
    // Send the email
    let result = mailer.send(email);
    
    assert!(result.is_ok());
}
```

#### Complete example

```rust,no_run
extern crate lettre;

use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::{SendableEmail, Envelope, EmailAddress, Transport, SmtpClient};
use lettre::smtp::extension::ClientId;
use lettre::smtp::ConnectionReuseParameters;

fn main() {
    let email_1 = SendableEmail::new(
        Envelope::new(
            Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
            vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
        ).unwrap(),
        "id1".to_string(),
        "Hello world".to_string().into_bytes(),
    );
    
    let email_2 = SendableEmail::new(
        Envelope::new(
            Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
            vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
        ).unwrap(),
        "id2".to_string(),
        "Hello world a second time".to_string().into_bytes(),
    );
    
    // Connect to a remote server on a custom port
    let mut mailer = SmtpClient::new_simple("server.tld").unwrap()
        // Set the name sent during EHLO/HELO, default is `localhost`
        .hello_name(ClientId::Domain("my.hostname.tld".to_string()))
        // Add credentials for authentication
        .credentials(Credentials::new("username".to_string(), "password".to_string()))
        // Enable SMTPUTF8 if the server supports it
        .smtp_utf8(true)
        // Configure expected authentication mechanism
        .authentication_mechanism(Mechanism::Plain)
        // Enable connection reuse
        .connection_reuse(ConnectionReuseParameters::ReuseUnlimited).transport();
    
    let result_1 = mailer.send(email_1);
    assert!(result_1.is_ok());
    
    // The second email will use the same connection
    let result_2 = mailer.send(email_2);
    assert!(result_2.is_ok());
    
    // Explicitly close the SMTP transaction as we enabled connection reuse
    mailer.close();
}
```

You can specify custom TLS settings:

```rust,no_run
extern crate native_tls;
extern crate lettre;

use lettre::{
    ClientSecurity, ClientTlsParameters, EmailAddress, Envelope, 
    SendableEmail, SmtpClient, Transport,
};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::ConnectionReuseParameters;
use native_tls::{Protocol, TlsConnector};

fn main() {
    let email = SendableEmail::new(
        Envelope::new(
            Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
            vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
        ).unwrap(),
        "message_id".to_string(),
        "Hello world".to_string().into_bytes(),
    );

    let mut tls_builder = TlsConnector::builder();
    tls_builder.min_protocol_version(Some(Protocol::Tlsv10));
    let tls_parameters =
        ClientTlsParameters::new(
            "smtp.example.com".to_string(),
            tls_builder.build().unwrap()
        );

    let mut mailer = SmtpClient::new(
        ("smtp.example.com", 465), ClientSecurity::Wrapper(tls_parameters)
    ).unwrap()
        .authentication_mechanism(Mechanism::Login)
        .credentials(Credentials::new(
            "example_username".to_string(), "example_password".to_string()
        ))
        .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
        .transport();

    let result = mailer.send(email);

    assert!(result.is_ok());

    mailer.close();
}
```

#### Lower level

You can also send commands, here is a simple email transaction without
error handling:

```rust,no_run
extern crate lettre;

use lettre::EmailAddress;
use lettre::smtp::SMTP_PORT;
use lettre::smtp::client::InnerClient;
use lettre::smtp::client::net::NetworkStream;
use lettre::smtp::extension::ClientId;
use lettre::smtp::commands::*;

fn main() {
    let mut email_client: InnerClient<NetworkStream> = InnerClient::new();
    let _ = email_client.connect(&("localhost", SMTP_PORT), None, None);
    let _ = email_client.command(EhloCommand::new(ClientId::new("my_hostname".to_string())));
    let _ = email_client.command(
                MailCommand::new(Some(EmailAddress::new("user@example.com".to_string()).unwrap()), vec![])
            );
    let _ = email_client.command(
                RcptCommand::new(EmailAddress::new("user@example.org".to_string()).unwrap(), vec![])
            );
    let _ = email_client.command(DataCommand);
    let _ = email_client.message(Box::new("Test email".as_bytes()));
    let _ = email_client.command(QuitCommand);
}
```

