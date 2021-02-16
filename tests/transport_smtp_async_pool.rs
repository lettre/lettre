#[cfg(all(test, feature = "smtp-transport", feature = "tokio1-pool"))]
mod test {
    use tokio1_crate as tokio;
    use lettre::{address::Envelope, AsyncSmtpTransport, Tokio1Connector, Tokio1Transport};

    fn envelope() -> Envelope {
        Envelope::new(
            Some("user@localhost".parse().unwrap()),
            vec!["root@localhost".parse().unwrap()],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn send_one_async() {
        let mailer = AsyncSmtpTransport::<Tokio1Connector>::builder_dangerous("127.0.0.1")
            .port(2525)
            .build();

        let result = mailer.send_raw(&envelope(), b"async test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn send_multiple_async() {
        let mailer = AsyncSmtpTransport::<Tokio1Connector>::builder_dangerous("127.0.0.1")
            .port(2525)
            .build();

        let mut handles = Vec::new();

        for i in 0..2 {
            let mcopy = mailer.clone();
            let handle = tokio::spawn(async move {
                let raw_msg = format!("async test {}", i);
                let result = mcopy.send_raw(&envelope(), raw_msg.as_bytes()).await;
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        let (first, second) = tokio::join!(handles.pop().unwrap(), handles.pop().unwrap());
        assert!(first.is_ok());
        assert!(second.is_ok());
    }
}
