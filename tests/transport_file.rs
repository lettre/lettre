extern crate lettre;

use lettre::transport::file::FileEmailTransport;
use lettre::transport::EmailTransport;
use lettre::email::EmailBuilder;

#[test]
fn file_transport() {
    let mut sender = FileEmailTransport::new("/tmp/");
    let email = EmailBuilder::new()
                    .to("root@localhost")
                    .from("user@localhost")
                    .body("Hello World!")
                    .subject("Hello")
                    .build()
                    .unwrap();
    let result = sender.send(email);
    assert!(result.is_ok());
}
