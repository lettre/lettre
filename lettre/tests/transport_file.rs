extern crate lettre;

#[cfg(test)]
#[cfg(feature = "file-transport")]
mod test {

    use lettre::file::FileTransport;
    use lettre::{EmailAddress, Envelope, SendableEmail, Transport};
    use std::env::temp_dir;
    use std::fs::remove_file;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn file_transport() {
        let mut sender = FileTransport::new(temp_dir());
        let email = SendableEmail::new(
            Envelope::new(
                Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
                vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
            ).unwrap(),
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
