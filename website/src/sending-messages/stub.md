#### Stub Transport

The stub transport only logs message envelope and drops the content. It can be useful for
testing purposes.

```rust
extern crate lettre;

use lettre::stub::StubTransport;
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

    let mut sender = StubTransport::new_positive();
    let result = sender.send(email);
    assert!(result.is_ok());
}
```

Will log (when using a logger like `env_logger`):

```text
b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
```
