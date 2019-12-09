#### Sendmail Transport

The sendmail transport sends the email using the local sendmail command.

```rust,no_run
# #[cfg(feature = "transport-sendmail")]
# {
extern crate lettre;

use lettre::sendmail::SendmailTransport;
use lettre::{SendableEmail, Envelope, EmailAddress, Transport};

fn main() {
    let email = SendableEmail::new(
        Envelope::new(
            Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
            vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
        ).unwrap(),
        "id".to_string(),
        "Hello world".to_string().into_bytes(),
    );
    
    let mut sender = SendmailTransport::new();
    let result = sender.send(email);
    assert!(result.is_ok());
}
# }
```
