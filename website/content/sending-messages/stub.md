+++
date = "2017-05-21T23:46:17+02:00"
title = "Stub transport"
toc = true
weight = 5

+++

The stub transport only logs message envelope and drops the content. It can be useful for
testing purposes.

``` rust
use lettre::stub::StubEmailTransport;
use lettre::{SimpleSendableEmail, EmailTransport};

let email = SimpleSendableEmail::new(
                "user@localhost",
                vec!["root@localhost"],
                "message_id",
                "Hello world"
            );

let mut sender = StubEmailTransport;
let result = sender.send(email);
assert!(result.is_ok());
```

Will log the line:

```text
b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
```