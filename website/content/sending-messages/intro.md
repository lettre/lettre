+++
date = "2017-05-21T23:46:17+02:00"
title = "Introduction"
toc = true
weight = 1

+++

This mailer contains several different transports for your emails. To be sendable, the
emails have to implement `SendableEmail`, which is the case for emails created with `lettre_email`.

The following transports are available:

* The `SmtpTransport` uses the SMTP protocol to send the message over the network. It is
  the prefered way of sending emails.
* The `SendmailTransport` uses the sendmail command to send messages. It is an alternative to
  the SMTP transport.
* The `FileTransport` creates a file containing the email content to be sent. It can be used
  for debugging or if you want to keep all sent emails.
* The `StubTransport` is useful for debugging, and only prints the content of the email in the
  logs.
