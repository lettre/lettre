extern crate lettre;

#[cfg(test)]
#[cfg(feature = "smtp-transport")]
mod test {

    use lettre::{ClientSecurity, EmailTransport, SimpleSendableEmail, SmtpTransport};

    #[test]
    fn smtp_transport_simple() {
        let mut sender = SmtpTransport::builder("127.0.0.1:2525", ClientSecurity::None)
            .unwrap()
            .build();
        let email = SimpleSendableEmail::new(
            "user@localhost".to_string(),
            &["root@localhost".to_string()],
            "smtp_id".to_string(),
            "Hello smtp".to_string(),
        ).unwrap();

        sender.send(&email).unwrap();
    }

}
