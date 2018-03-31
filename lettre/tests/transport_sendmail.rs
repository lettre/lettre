extern crate lettre;

#[cfg(test)]
#[cfg(feature = "sendmail-transport")]
mod test {

    use lettre::{EmailTransport, SimpleSendableEmail};
    use lettre::sendmail::SendmailTransport;

    #[test]
    fn sendmail_transport_simple() {
        let mut sender = SendmailTransport::new();
        let email = SimpleSendableEmail::new(
            "user@localhost".to_string(),
            &["root@localhost".to_string()],
            "sendmail_id".to_string(),
            "Hello sendmail".to_string(),
        ).unwrap();

        let result = sender.send(&email);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

}
