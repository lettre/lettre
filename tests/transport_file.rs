extern crate lettre;

use lettre::email::{EmailBuilder, SendableEmail};
use lettre::transport::EmailTransport;

use lettre::transport::file::FileEmailTransport;
use std::env::temp_dir;
use std::fs::File;
use std::fs::remove_file;
use std::io::Read;

#[test]
fn file_transport() {
    let mut sender = FileEmailTransport::new(temp_dir());
    let email = EmailBuilder::new()
        .to("root@localhost")
        .from("user@localhost")
        .body("Hello World!")
        .subject("Hello file")
        .build()
        .unwrap();
    let result = sender.send(email.clone());
    assert!(result.is_ok());

    let message_id = email.message_id();
    let file = format!("{}/{}.txt", temp_dir().to_str().unwrap(), message_id);
    let mut f = File::open(file.clone()).unwrap();
    let mut buffer = String::new();
    let _ = f.read_to_string(&mut buffer);

    assert_eq!(buffer,
               format!("{}: from=<user@localhost> to=<root@localhost>\n{}",
                       message_id,
                       email.message()));

    remove_file(file).unwrap();
}
