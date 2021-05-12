#[cfg(test)]
#[cfg(all(feature = "sendmail-transport", feature = "builder"))]
mod sync {
    use lettre::{Message, SendmailTransport, Transport};

    #[test]
    fn sendmail_transport() {
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
}

#[cfg(test)]
#[cfg(all(
    feature = "sendmail-transport",
    feature = "builder",
    feature = "tokio1"
))]
mod tokio_1 {
    use lettre::{AsyncSendmailTransport, AsyncTransport, Message, Tokio1Executor};
    use tokio1_crate as tokio;

    #[tokio::test]
    async fn sendmail_transport_tokio1() {
        let sender = AsyncSendmailTransport::<Tokio1Executor>::new();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .body(String::from("Be happy!"))
            .unwrap();

        let result = sender.send(email).await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
#[cfg(all(
    feature = "sendmail-transport",
    feature = "builder",
    feature = "async-std1"
))]
mod asyncstd_1 {
    use lettre::{AsyncSendmailTransport, AsyncStd1Executor, AsyncTransport, Message};

    #[async_std::test]
    async fn sendmail_transport_asyncstd1() {
        let sender = AsyncSendmailTransport::<AsyncStd1Executor>::new();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .body(String::from("Be happy!"))
            .unwrap();

        let result = sender.send(email).await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
