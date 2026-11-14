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
#[cfg(feature = "smtp-transport")]
mod read_response_sync {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::mpsc,
        thread,
        time::Duration,
    };

    use lettre::{
        SmtpTransport,
        transport::smtp::{
            client::SmtpConnection,
            commands::{Data, Mail, Quit, Rcpt},
            extension::ClientId,
        },
    };

    #[test]
    fn long_reply_line_succeeds() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut writer = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream);
            writer.write_all(b"220 test ESMTP\r\n").unwrap();
            let mut line = String::new();
            reader.read_line(&mut line).unwrap(); // EHLO
            let reply = format!("250 {}\r\n", "x".repeat(1_094));
            writer.write_all(reply.as_bytes()).unwrap();
            line.clear();
            let _ = reader.read_line(&mut line); // NOOP or QUIT
            let _ = writer.write_all(b"250 ok\r\n");
            line.clear();
            let _ = reader.read_line(&mut line); // QUIT
            let _ = writer.write_all(b"221 bye\r\n");
            let _ = writer.flush();
            line.clear();
            let _ = reader.read_line(&mut line);
        });

        let mailer = SmtpTransport::builder_dangerous("127.0.0.1")
            .port(port)
            .build();
        assert!(mailer.test_connection().unwrap());
    }

    #[test]
    fn long_error_reply_after_data_preserves_status() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let detail = "y".repeat(4_096);
        let expected_tail = detail.clone();
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut writer = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream);
            writer.write_all(b"220 test ESMTP\r\n").unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).unwrap(); // EHLO
            writer.write_all(b"250 test\r\n").unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap(); // MAIL FROM
            writer.write_all(b"250 2.1.0 Ok\r\n").unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap(); // RCPT TO
            writer.write_all(b"250 2.1.5 Ok\r\n").unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap(); // DATA
            writer
                .write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n")
                .unwrap();

            loop {
                line.clear();
                reader.read_line(&mut line).unwrap();
                if line == ".\r\n" {
                    break;
                }
            }
            let refusal = format!("554 5.2.252 SendAsDenied; {detail}\r\n");
            writer.write_all(refusal.as_bytes()).unwrap();

            line.clear();
            if reader.read_line(&mut line).is_ok() {
                let timeout = format!("451 4.7.1 Try again later; {detail}\r\n");
                let _ = writer.write_all(timeout.as_bytes());
            }
            line.clear();
            let _ = reader.read_line(&mut line);
            let _ = writer.write_all(b"221 bye\r\n");
            let _ = writer.flush();
            line.clear();
            let _ = reader.read_line(&mut line);
        });

        let mut client = SmtpConnection::connect(
            addr,
            Some(Duration::from_secs(5)),
            &ClientId::Domain("test".into()),
            None,
            None,
        )
        .unwrap();
        client
            .command(Mail::new(
                Some("sender@example.com".parse().unwrap()),
                vec![],
            ))
            .unwrap();
        client
            .command(Rcpt::new("rcpt@example.com".parse().unwrap(), vec![]))
            .unwrap();
        client.command(Data).unwrap();

        let err = client
            .message(b"Subject: test\r\n\r\nhello")
            .expect_err("554 refusal must return an error");
        assert!(err.is_permanent(), "expected permanent error, got {err:?}");
        assert!(!err.is_response());
        assert_eq!(err.status().map(u16::from), Some(554));
        assert!(err.to_string().contains(&expected_tail));

        let transient = client
            .command(Mail::new(
                Some("sender@example.com".parse().unwrap()),
                vec![],
            ))
            .expect_err("451 refusal must return an error");
        assert!(
            transient.is_transient(),
            "expected transient error, got {transient:?}"
        );
        assert_eq!(transient.status().map(u16::from), Some(451));
        let _ = client.command(Quit);
    }

    #[test]
    fn oversized_line_without_newline_is_bounded() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        thread::spawn(move || {
            if let Ok((mut sock, _)) = listener.accept() {
                let mut payload = b"220 ".to_vec();
                payload.resize(100_001, b'x');
                let _ = sock.write_all(&payload);
                let _ = sock.flush();
                let _ = done_rx.recv_timeout(Duration::from_secs(5));
            }
        });

        let err = SmtpConnection::connect(
            addr,
            Some(Duration::from_secs(5)),
            &ClientId::Domain("test".into()),
            None,
            None,
        )
        .err()
        .expect("oversized line must surface as an error");
        let _ = done_tx.send(());
        assert!(
            err.is_response(),
            "expected response-kind error, got {err:?}"
        );
    }
}

#[cfg(test)]
#[cfg(all(feature = "smtp-transport", feature = "tokio1"))]
mod read_response_caps {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::mpsc,
        thread,
        time::Duration,
    };

    use lettre::transport::smtp::{
        client::AsyncSmtpConnection,
        commands::{Data, Mail, Rcpt},
        extension::ClientId,
    };
    use tokio1_crate as tokio;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn oversized_line_is_bounded() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        thread::spawn(move || {
            if let Ok((mut sock, _)) = listener.accept() {
                let mut line = b"220 ".to_vec();
                line.resize(100_001, b'x');
                let _ = sock.write_all(&line);
                let _ = sock.flush();
                let _ = done_rx.recv_timeout(Duration::from_secs(5));
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
        let _ = done_tx.send(());

        let err = match result {
            Ok(_) => panic!("oversized line must surface as an error"),
            Err(e) => e,
        };
        assert!(
            err.is_response(),
            "expected response-kind error, got {err:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn oversized_multiline_is_bounded() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut sock, _)) = listener.accept() {
                let line = format!("220-{}\r\n", "x".repeat(2_000));
                for _ in 0..60 {
                    if sock.write_all(line.as_bytes()).is_err() {
                        break;
                    }
                }
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
            Ok(_) => panic!("oversized multiline response must surface as an error"),
            Err(e) => e,
        };
        assert!(
            err.is_response(),
            "expected response-kind error, got {err:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn long_reply_and_error_lines_preserve_status() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let detail = "z".repeat(16_000);
        let expected_tail = detail.clone();
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut writer = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream);
            let banner = format!("220 {}\r\n", "b".repeat(1_500));
            writer.write_all(banner.as_bytes()).unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).unwrap(); // EHLO
            let ehlo = format!(
                "250-mail.example.com {}\r\n250 8BITMIME\r\n",
                "e".repeat(2_000)
            );
            writer.write_all(ehlo.as_bytes()).unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap(); // MAIL FROM
            writer.write_all(b"250 2.1.0 Ok\r\n").unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap(); // RCPT TO
            writer.write_all(b"250 2.1.5 Ok\r\n").unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap(); // DATA
            writer
                .write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n")
                .unwrap();

            loop {
                line.clear();
                reader.read_line(&mut line).unwrap();
                if line == ".\r\n" {
                    break;
                }
            }
            let refusal = format!("554 5.2.252 SendAsDenied; {detail}\r\n");
            writer.write_all(refusal.as_bytes()).unwrap();
            writer.flush().unwrap();
            line.clear();
            let _ = reader.read_line(&mut line);
        });

        let mut client = AsyncSmtpConnection::connect_tokio1(
            addr,
            Some(Duration::from_secs(5)),
            &ClientId::Domain("test".into()),
            None,
            None,
        )
        .await
        .unwrap();
        client
            .command(Mail::new(
                Some("sender@example.com".parse().unwrap()),
                vec![],
            ))
            .await
            .unwrap();
        client
            .command(Rcpt::new("rcpt@example.com".parse().unwrap(), vec![]))
            .await
            .unwrap();
        client.command(Data).await.unwrap();

        let err = client
            .message(b"Subject: test\r\n\r\nhello")
            .await
            .expect_err("554 refusal must return an error");
        assert!(err.is_permanent(), "expected permanent error, got {err:?}");
        assert!(!err.is_response());
        assert_eq!(err.status().map(u16::from), Some(554));
        assert!(err.to_string().contains(&expected_tail));
    }
}
