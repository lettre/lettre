#[cfg(test)]
#[cfg(feature = "smtp-transport")]
mod sync {
    use lettre::{Message, SmtpTransport, Transport};

    #[cfg(feature = "builder")]
    #[test]
    fn smtp_transport_simple() {
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .body(String::from("Be happy!"))
            .unwrap();

        let sender = SmtpTransport::builder_dangerous("127.0.0.1")
            .port(2525)
            .build();
        sender.send(&email).unwrap();
    }

    #[test]
    fn smtp_transport_clone() {
        let sender = SmtpTransport::builder_dangerous("127.0.0.1")
            .port(2525)
            .build();

        let _ = sender.clone();
    }
}

#[cfg(test)]
#[cfg(all(feature = "smtp-transport", feature = "tokio1"))]
mod tokio_1 {
    use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
    use tokio1_crate as tokio;

    #[cfg(feature = "builder")]
    #[tokio::test]
    async fn smtp_transport_simple_tokio1() {
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .body(String::from("Be happy!"))
            .unwrap();

        let sender: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("127.0.0.1")
                .port(2525)
                .build();
        sender.send(email).await.unwrap();
    }

    #[tokio::test]
    async fn smtp_transport_clone() {
        let sender: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("127.0.0.1")
                .port(2525)
                .build();

        let _ = sender.clone();
    }
}

#[cfg(test)]
#[cfg(all(feature = "smtp-transport", feature = "async-std1"))]
mod asyncstd_1 {
    use lettre::{AsyncSmtpTransport, AsyncStd1Executor, AsyncTransport, Message};

    #[cfg(feature = "builder")]
    #[async_std::test]
    async fn smtp_transport_simple_asyncstd1() {
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .body(String::from("Be happy!"))
            .unwrap();

        let sender: AsyncSmtpTransport<AsyncStd1Executor> =
            AsyncSmtpTransport::<AsyncStd1Executor>::builder_dangerous("127.0.0.1")
                .port(2525)
                .build();
        sender.send(email).await.unwrap();
    }

    #[async_std::test]
    async fn smtp_transport_clone() {
        let sender: AsyncSmtpTransport<AsyncStd1Executor> =
            AsyncSmtpTransport::<AsyncStd1Executor>::builder_dangerous("127.0.0.1")
                .port(2525)
                .build();

        let _ = sender.clone();
    }
}
