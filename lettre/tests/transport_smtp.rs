#[cfg(test)]
#[cfg(feature = "smtp-transport")]
mod test {
    use lettre::{ClientSecurity, EmailAddress, Envelope, SendableEmail, SmtpClient, Transport};

    #[test]
    fn smtp_transport_simple() {
        let email = SendableEmail::new(
            Envelope::new(
                Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
                vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
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
