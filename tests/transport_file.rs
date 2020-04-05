#[cfg(test)]
#[cfg(feature = "file-transport")]
mod test {
    use lettre::transport::file::FileTransport;
    use lettre::{Message, Transport};
    use std::env::temp_dir;
    use std::fs::remove_file;
    use std::fs::File;
    use std::io::Read;

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
            "{\"envelope\":{\"forward_path\":[\"hei@domain.tld\"],\"reverse_path\":\"nobody@domain.tld\"},\"message\":[70,114,111,109,58,32,78,111,66,111,100,121,32,60,110,111,98,111,100,121,64,100,111,109,97,105,110,46,116,108,100,62,13,10,82,101,112,108,121,45,84,111,58,32,89,117,105,110,32,60,121,117,105,110,64,100,111,109,97,105,110,46,116,108,100,62,13,10,84,111,58,32,72,101,105,32,60,104,101,105,64,100,111,109,97,105,110,46,116,108,100,62,13,10,83,117,98,106,101,99,116,58,32,72,97,112,112,121,32,110,101,119,32,121,101,97,114,13,10,13,10,66,101,32,104,97,112,112,121,33]}");
        remove_file(file).unwrap();
    }
}
