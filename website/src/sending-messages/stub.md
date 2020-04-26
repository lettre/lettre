#### Stub Transport

The stub transport only logs message envelope and drops the content. It can be useful for
testing purposes.

```rust
# extern crate lettre;

use lettre::transport::stub::StubTransport;
use lettre::{Message, Envelope, Transport};

fn main() {
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body("Be happy!")
        .unwrap();

    let mut sender = StubTransport::new_positive();
    let result = sender.send(&email);
    assert!(result.is_ok());
}
```

Will log (when using a logger like `env_logger`):

```text
b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
```
