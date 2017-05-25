+++
date = "2017-05-21T23:46:17+02:00"
title = "File transport"
toc = true
weight = 4

+++

The file transport writes the emails to the given directory. The name of the file will be
`message_id.txt`.
It can be useful for testing purposes, or if you want to keep track of sent messages.

``` rust
use std::env::temp_dir;

use lettre::file::FileEmailTransport;
use lettre::{SimpleSendableEmail, EmailTransport};

// Write to the local temp directory
let mut sender = FileEmailTransport::new(temp_dir());
let email = SimpleSendableEmail::new(
                "user@localhost",
                vec!["root@localhost"],
                "message_id",
                "Hello world"
            );

let result = sender.send(email);
assert!(result.is_ok());
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
