extern crate lettre;

#[cfg(test)]
#[cfg(feature = "smtp-transport")]
mod test {
    use lettre::{ClientSecurity, EmailAddress, Envelope, SendableEmail, SmtpTransport, Transport};

    #[test]
    fn smtp_transport_simple() {
        let mut sender = SmtpTransport::builder("127.0.0.1:2525", ClientSecurity::None)
            .unwrap()
            .build();
        let email = SendableEmail::new(
            Envelope::new(
                Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
                vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
            ).unwrap(),
            "id".to_string(),
            "Hello ß☺ example".to_string().into_bytes(),
        );

        sender.send(email).unwrap();
    }

}
