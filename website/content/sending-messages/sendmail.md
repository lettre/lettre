+++
date = "2017-05-21T23:46:17+02:00"
title = "Sendmail transport"
toc = true
weight = 3

+++

The sendmail transport sends the email using the local sendmail command.

{{< highlight rust >}}
use lettre::sendmail::SendmailTransport;
use lettre::{SimpleSendableEmail, EmailTransport};

let email = SimpleSendableEmail::new(
                "user@localhost",
                vec!["root@localhost"],
                "message_id",
                "Hello world"
            );

let mut sender = SendmailTransport::new();
let result = sender.send(email);
assert!(result.is_ok());
{{< /highlight >}}
