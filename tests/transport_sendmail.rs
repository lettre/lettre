#[cfg(test)]
#[cfg(feature = "sendmail-transport")]
mod test {
    use lettre::sendmail::SendmailTransport;
    use lettre::{Address, Email, Envelope, Transport};
    use std::str::FromStr;

    #[test]
    fn sendmail_transport_simple() {
        let mut sender = SendmailTransport::new();
        let email = Email::new(
            Envelope::new(
                Some(Address::from_str("user@localhost").unwrap()),
                vec![Address::from_str("root@localhost").unwrap()],
            )
            .unwrap(),
            "id".to_string(),
            "Hello ß☺ example".to_string().into_bytes(),
        );

        let result = sender.send(email);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
