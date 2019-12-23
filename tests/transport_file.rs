#[cfg(test)]
#[cfg(feature = "file-transport")]
mod test {
    use lettre::file::FileTransport;
    use lettre::{Address, Email, Envelope, Transport};
    use std::env::temp_dir;
    use std::fs::remove_file;
    use std::fs::File;
    use std::io::Read;
    use std::str::FromStr;

    #[test]
    fn file_transport() {
        let mut sender = FileTransport::new(temp_dir());
        let email = Email::new(
            Envelope::new(
                Some(Address::from_str("user@localhost").unwrap()),
                vec![Address::from_str("root@localhost").unwrap()],
            )
            .unwrap(),
            "id".to_string(),
            "Hello ß☺ example".to_string().into_bytes(),
        );
        let message_id = email.message_id().to_string();

        let result = sender.send(email);
        assert!(result.is_ok());

        let file = format!("{}/{}.json", temp_dir().to_str().unwrap(), message_id);
        let mut f = File::open(file.clone()).unwrap();
        let mut buffer = String::new();
        let _ = f.read_to_string(&mut buffer);

        assert_eq!(
            buffer,
            "{\"envelope\":{\"forward_path\":[\"root@localhost\"],\"reverse_path\":\"user@localhost\"},\"message_id\":\"id\",\"message\":[72,101,108,108,111,32,195,159,226,152,186,32,101,120,97,109,112,108,101]}"
        );

        remove_file(file).unwrap();
    }
}
