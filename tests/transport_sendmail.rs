#[cfg(test)]
#[cfg(all(feature = "sendmail-transport", feature = "builder"))]
mod test {
    use lettre::{transport::sendmail::SendmailTransport, Message};

    #[cfg(feature = "tokio02")]
    use tokio02_crate as tokio;

    #[test]
    fn sendmail_transport() {
        use lettre::Transport;
        let sender = SendmailTransport::new();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .body(String::from("Be happy!"))
            .unwrap();

        let result = sender.send(&email);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[cfg(feature = "async-std1")]
    #[async_attributes::test]
    async fn sendmail_transport_asyncstd1() {
        use lettre::AsyncStd1Transport;

        let sender = SendmailTransport::new();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
            .body(String::from("Be happy!"))
            .unwrap();

        let result = sender.send(email).await;
        assert!(result.is_ok());
    }

    #[cfg(feature = "tokio02")]
    #[tokio::test]
    async fn sendmail_transport_tokio02() {
        use lettre::Tokio02Transport;

        let sender = SendmailTransport::new();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
            .body(String::from("Be happy!"))
            .unwrap();

        let result = sender.send(email).await;
        assert!(result.is_ok());
    }
}
