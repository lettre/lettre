#### Sendmail Transport

The sendmail transport sends the email using the local sendmail command.

```rust,no_run
# #[cfg(feature = "sendmail-transport")]
# {
# extern crate lettre;

use lettre::transport::sendmail::SendmailTransport;
use lettre::{Message, Envelope, EmailAddress, Transport};

fn main() {
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body("Be happy!")
        .unwrap();

    let mut sender = SendmailTransport::new();
    let result = sender.send(email);
    assert!(result.is_ok());
}
# }
```
