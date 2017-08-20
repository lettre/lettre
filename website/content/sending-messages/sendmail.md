+++
date = "2017-05-21T23:46:17+02:00"
title = "Sendmail transport"
toc = true
weight = 3

+++

The sendmail transport sends the email using the local sendmail command.

``` rust
use lettre::sendmail::SendmailTransport;
use lettre::{SimpleSendableEmail, EmailTransport, EmailAddress};

let email = SimpleSendableEmail::new(
                EmailAddress::new("user@localhost".to_string()),
                vec![EmailAddress::new("root@localhost".to_string())],
                "message_id".to_string(),
                "Hello world".to_string(),
            );

let mut sender = SendmailTransport::new();
let result = sender.send(&email);
assert!(result.is_ok());
```
