#[cfg(test)]
#[cfg(feature = "sendmail-transport")]
mod test {
    use lettre::{transport::sendmail::SendmailTransport, Message, Transport};

    #[test]
    fn sendmail_transport_simple() {
        let mut sender = SendmailTransport::new();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .body("Be happy!")
            .unwrap();

        let result = sender.send(&email);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
