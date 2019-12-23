#[cfg(test)]
#[cfg(feature = "smtp-transport")]
mod test {
    use lettre::{Address, ClientSecurity, Email, Envelope, SmtpClient, Transport};
    use std::str::FromStr;

    #[test]
    fn smtp_transport_simple() {
        let email = Email::new(
            Envelope::new(
                Some(Address::from_str("user@localhost").unwrap()),
                vec![Address::from_str("root@localhost").unwrap()],
            )
            .unwrap(),
            "id".to_string(),
            "Hello ß☺ example".to_string().into_bytes(),
        );

        SmtpClient::new("127.0.0.1:2525", ClientSecurity::None)
            .unwrap()
            .transport()
            .send(email)
            .unwrap();
    }
}
