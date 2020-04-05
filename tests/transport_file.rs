#[cfg(test)]
#[cfg(feature = "file-transport")]
mod test {
    use lettre::file::FileTransport;
    use lettre::{Address, Envelope, Message, Transport};
    use std::env::temp_dir;
    use std::fs::remove_file;
    use std::fs::File;
    use std::io::Read;
    use std::str::FromStr;

    #[test]
    fn file_transport() {
        let mut sender = FileTransport::new(temp_dir());
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .body("Be happy!")
            .unwrap();

        let result = sender.send(email);
        let id = result.unwrap();

        let file = temp_dir().join(format!("{}.json", id));
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
