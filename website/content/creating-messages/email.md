### Creating messages

This section explains how to create emails.

#### Simple example

The `email` part builds email messages. For now, it does not support attachments.
An email is built using an `EmailBuilder`. The simplest email could be:

```rust
extern crate lettre_email;

use lettre_email::EmailBuilder;

fn main() {
    // Create an email
    let email = EmailBuilder::new()
        // Addresses can be specified by the tuple (email, alias)
        .to(("user@example.org", "Firstname Lastname"))
        // ... or by an address only
        .from("user@example.com")
        .subject("Hi, Hello world")
        .text("Hello world.")
        .build();
    
    assert!(email.is_ok());
}
```

When the `build` method is called, the `EmailBuilder` will add the missing headers (like
`Message-ID` or `Date`) and check for missing necessary ones (like `From` or `To`). It will
then generate an `Email` that can be sent.

The `text()` method will create a plain text email, while the `html()` method will create an
HTML email. You can use the `alternative()` method to provide both versions, using plain text
as fallback for the HTML version.

#### Complete example

Below is a more complete example, not using method chaining:

```rust
extern crate lettre_email;

use lettre_email::EmailBuilder;

fn main() {
    let mut builder = EmailBuilder::new();
    builder.add_to(("user@example.org", "Alias name"));
    builder.add_cc(("user@example.net", "Alias name"));
    builder.add_from("no-reply@example.com");
    builder.add_from("no-reply@example.eu");
    builder.set_sender("no-reply@example.com");
    builder.set_subject("Hello world");
    builder.set_alternative("<h2>Hi, Hello world.</h2>", "Hi, Hello world.");
    builder.add_reply_to("contact@example.com");
    builder.add_header(("X-Custom-Header", "my header"));
    
    let email = builder.build();
    assert!(email.is_ok());
}
```

See the `EmailBuilder` documentation for a complete list of methods.

