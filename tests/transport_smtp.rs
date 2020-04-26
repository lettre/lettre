#[cfg(test)]
#[cfg(feature = "smtp-transport")]
mod test {
    use lettre::{ClientSecurity, Message, SmtpClient, Transport};

    #[test]
    fn smtp_transport_simple() {
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .body("Be happy!")
            .unwrap();
        SmtpClient::new("127.0.0.1:2525", ClientSecurity::None)
            .unwrap()
            .transport()
            .send(&email)
            .unwrap();
    }
}
