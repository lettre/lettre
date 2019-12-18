#[cfg(test)]
#[cfg(feature = "sendmail-transport")]
mod test {
    use lettre::sendmail::SendmailTransport;
    use lettre::{Email, EmailAddress, Envelope, Transport};

    #[test]
    fn sendmail_transport_simple() {
        let mut sender = SendmailTransport::new();
        let email = Email::new(
            Envelope::new(
                Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
                vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
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
