#[cfg(test)]
#[cfg(all(feature = "smtp-transport", feature = "builder"))]
mod sync {
    use lettre::{Message, SmtpTransport, Transport};

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
}

#[cfg(test)]
#[cfg(all(feature = "smtp-transport", feature = "builder", feature = "tokio1"))]
mod tokio_1 {
    use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
    use tokio1_crate as tokio;

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
}

#[cfg(test)]
#[cfg(all(
    feature = "smtp-transport",
    feature = "builder",
    feature = "async-std1"
))]
mod asyncstd_1 {
    use lettre::{AsyncSmtpTransport, AsyncStd1Executor, AsyncTransport, Message};

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
}

#[cfg(test)]
#[cfg(all(feature = "smtp-transport", feature = "tokio1"))]
mod read_response_caps {
    use std::{io::Write, net::TcpListener, thread, time::Duration};

    use lettre::transport::smtp::{client::AsyncSmtpConnection, extension::ClientId};
    use tokio1_crate as tokio;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn oversized_line_is_bounded() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut sock, _)) = listener.accept() {
                let mut line = vec![b'x'; 4096];
                line.extend_from_slice(b"\r\n");
                let _ = sock.write_all(&line);
            }
        });

        let result = tokio::time::timeout(
            Duration::from_secs(5),
            AsyncSmtpConnection::connect_tokio1(
                addr,
                None,
                &ClientId::Domain("test".into()),
                None,
                None,
            ),
        )
        .await
        .expect("connect must return within 5s, not hang");

        let err = match result {
            Ok(_) => panic!("oversized line must surface as an error"),
            Err(e) => e,
        };
        assert!(
            err.is_response(),
            "expected response-kind error, got {err:?}"
        );
    }
}
