#### File Transport

The file transport writes the emails to the given directory. The name of the file will be
`message_id.txt`.
It can be useful for testing purposes, or if you want to keep track of sent messages.

```rust
# #[cfg(feature = "file-transport")]
# {
extern crate lettre;

use std::env::temp_dir;

use lettre::transport::file::FileTransport;
use lettre::{Transport, Envelope, EmailAddress, Message};

fn main() {
    // Write to the local temp directory
    let mut sender = FileTransport::new(temp_dir());
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body("Be happy!")
        .unwrap();

    let result = sender.send(email);
    assert!(result.is_ok());
}
# }
```

Example result in `/tmp/b7c211bc-9811-45ce-8cd9-68eab575d695.txt`:

```text
b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
To: <root@localhost>
From: <user@localhost>
Subject: Hello
Date: Sat, 31 Oct 2015 13:42:19 +0100
Message-ID: <b7c211bc-9811-45ce-8cd9-68eab575d695.lettre@localhost>

Hello World!
```
