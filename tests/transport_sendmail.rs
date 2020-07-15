#[cfg(test)]
#[cfg(feature = "sendmail-transport")]
mod test {
    use lettre::{transport::sendmail::SendmailTransport, Message};

    #[test]
    fn sendmail_transport() {
        use lettre::Transport;
        let sender = SendmailTransport::new();
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

    #[cfg(feature = "async")]
    #[async_attributes::test]
    async fn sendmail_transport_async() {
        use lettre::r#async::Transport;
        let sender = SendmailTransport::new();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
            .body("Be happy!")
            .unwrap();

        let result = sender.send(email).await;
        assert!(result.is_ok());
    }
}
