extern crate lettre;

use std::env::temp_dir;

use lettre::transport::file::FileEmailTransport;
use lettre::transport::EmailTransport;
use lettre::email::EmailBuilder;

#[test]
fn file_transport() {
    let mut sender = FileEmailTransport::new(temp_dir());
    let email = EmailBuilder::new()
                    .to("root@localhost")
                    .from("user@localhost")
                    .body("Hello World!")
                    .subject("Hello")
                    .build()
                    .unwrap();
    let result = sender.send(email);
    assert!(result.is_ok());

    message_id = email.message_id();

}
