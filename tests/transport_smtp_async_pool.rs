#[cfg(all(test, feature = "smtp-transport", feature = "tokio1-pool"))]
mod test {
    use lettre::{address::Envelope, AsyncSmtpTransport, Tokio1Connector, Tokio1Transport};
    use tokio1_crate as tokio;

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

    #[cfg(all(
        feature = "tokio1-pool",
        not(any(feature = "tokio02", feature = "async-std1"))
    ))]
    #[tokio::test]
    async fn send_one_with_pool_async() {
        use transport::smtp::AsyncPoolConfig;
        let cfg = AsyncPoolConfig::new().min_idle(1).max_size(2);

        let mailer = AsyncSmtpTransport::<Tokio1Connector>::builder_dangerous("127.0.0.1")
            .port(2525)
            .pool_config(cfg)
            .build();

        let result = mailer.send_raw(&envelope(), b"async test with pool").await;
        assert!(result.is_ok());
    }
}
