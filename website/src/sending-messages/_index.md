### Sending Messages

This section explains how to manipulate emails you have created.

This mailer contains several different transports for your emails. To be sendable, the
emails have to implement `Email`, which is the case for emails created with `lettre::builder`.

The following transports are available:

* The `SmtpTransport` uses the SMTP protocol to send the message over the network. It is
  the preferred way of sending emails.
* The `SendmailTransport` uses the sendmail command to send messages. It is an alternative to
  the SMTP transport.
* The `FileTransport` creates a file containing the email content to be sent. It can be used
  for debugging or if you want to keep all sent emails.
* The `StubTransport` is useful for debugging, and only prints the content of the email in the
  logs.
