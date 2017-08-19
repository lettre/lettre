extern crate lettre;

#[cfg(test)]
#[cfg(feature = "file-transport")]
mod test {

    use lettre::{EmailAddress, EmailTransport, SendableEmail, SimpleSendableEmail};
    #[cfg(feature = "file-transport")]
    use lettre::file::FileEmailTransport;
    use std::env::temp_dir;
    use std::fs::File;
    use std::fs::remove_file;
    use std::io::Read;

    #[test]
    #[cfg(feature = "file-transport")]
    fn file_transport() {
        let mut sender = FileEmailTransport::new(temp_dir());
        let email = SimpleSendableEmail::new(
            EmailAddress::new("user@localhost".to_string()),
            vec![EmailAddress::new("root@localhost".to_string())],
            "file_id".to_string(),
            "Hello file".to_string(),
        );
        let result = sender.send(&email);
        assert!(result.is_ok());

        let message_id = email.message_id();
        let file = format!("{}/{}.txt", temp_dir().to_str().unwrap(), message_id);
        let mut f = File::open(file.clone()).unwrap();
        let mut buffer = String::new();
        let _ = f.read_to_string(&mut buffer);

        assert_eq!(
            buffer,
            "{\"to\":[\"root@localhost\"],\"from\":\"user@localhost\",\"message_id\":\
            \"file_id\",\"message\":[72,101,108,108,111,32,102,105,108,101]}"
        );

        remove_file(file).unwrap();
    }
}
