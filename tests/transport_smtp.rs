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

// The TLS handshake of STARTTLS can fail, for example because the server
// closes the connection instead of starting it. The connection is closed
// from then on: using it, or closing it again, returns an error and must
// not panic.
#[cfg(test)]
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "native-tls", feature = "rustls", feature = "boring-tls")
))]
mod failed_tls_upgrade {
    use std::{
        io::{BufRead, BufReader, Write},
        net::{SocketAddr, TcpListener},
        thread,
        time::Duration,
    };

    use lettre::transport::smtp::{
        client::{SmtpConnection, TlsParameters},
        commands::Noop,
        extension::ClientId,
    };

    // Accepts STARTTLS, then closes the connection instead of doing the
    // TLS handshake
    fn server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut commands = BufReader::new(stream.try_clone().unwrap()).lines();
            stream.write_all(b"220 localhost ESMTP\r\n").unwrap();
            let _ehlo = commands.next();
            stream
                .write_all(b"250-localhost\r\n250 STARTTLS\r\n")
                .unwrap();
            let _starttls = commands.next();
            stream.write_all(b"220 go ahead\r\n").unwrap();
        });
        addr
    }

    #[test]
    fn closed_connection_does_not_panic() {
        let hello = ClientId::Domain("test".into());
        let mut conn =
            SmtpConnection::connect(server(), Some(Duration::from_secs(5)), &hello, None, None)
                .unwrap();
        let tls = TlsParameters::new("localhost".into()).unwrap();

        assert!(conn.starttls(&tls, &hello).is_err());
        assert!(conn.peer_certificate().is_err());
        assert!(conn.command(Noop).is_err());
        conn.abort();
    }

    #[cfg(all(
        feature = "tokio1",
        any(
            feature = "tokio1-native-tls",
            feature = "tokio1-rustls",
            feature = "tokio1-boring-tls"
        )
    ))]
    mod tokio_1 {
        use lettre::transport::smtp::client::AsyncSmtpConnection;
        use tokio1_crate as tokio;

        use super::*;

        #[tokio::test]
        async fn closed_connection_does_not_panic() {
            let hello = ClientId::Domain("test".into());
            let mut conn = AsyncSmtpConnection::connect_tokio1(
                server(),
                Some(Duration::from_secs(5)),
                &hello,
                None,
                None,
            )
            .await
            .unwrap();
            let tls = TlsParameters::new("localhost".into()).unwrap();

            assert!(conn.starttls(tls, &hello).await.is_err());
            assert!(conn.peer_certificate().is_err());
            assert!(conn.command(Noop).await.is_err());
            conn.abort().await;
        }
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

#[cfg(test)]
#[cfg(feature = "smtp-transport")]
mod data_terminator {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::TcpListener,
        thread,
        time::Duration,
    };

    use lettre::transport::smtp::{client::SmtpConnection, commands::Data, extension::ClientId};

    /// Sends `message` as mail data and returns the bytes the server received after `354`
    fn data_on_wire(message: &[u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (sock, _) = listener.accept().unwrap();
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut reader = BufReader::new(sock.try_clone().unwrap());
            let mut writer = sock;
            writer.write_all(b"220 ready\r\n").unwrap();
            reader.read_line(&mut String::new()).unwrap();
            writer.write_all(b"250 ok\r\n").unwrap();
            reader.read_line(&mut String::new()).unwrap();
            writer.write_all(b"354 go\r\n").unwrap();

            // the CRLF of the DATA command line counts as the end of a line
            let mut data = b"\r\n".to_vec();
            let mut byte = [0];
            while !data.ends_with(b"\r\n.\r\n") {
                reader.read_exact(&mut byte).unwrap();
                data.push(byte[0]);
            }
            writer.write_all(b"250 queued\r\n").unwrap();
            String::from_utf8(data.split_off(2)).unwrap()
        });

        let mut client = SmtpConnection::connect(
            addr,
            Some(Duration::from_secs(5)),
            &ClientId::Domain("test".into()),
            None,
            None,
        )
        .unwrap();
        client.command(Data).unwrap();
        client.message(message).unwrap();
        server.join().unwrap()
    }

    #[test]
    fn message_ending_with_crlf_gets_no_empty_line() {
        assert_eq!(
            data_on_wire(b"Subject: x\r\n\r\nbody\r\n.dot\r\n"),
            "Subject: x\r\n\r\nbody\r\n..dot\r\n.\r\n"
        );
    }

    #[test]
    fn message_without_final_crlf_gets_its_line_ended() {
        assert_eq!(
            data_on_wire(b"Subject: x\r\n\r\nbody"),
            "Subject: x\r\n\r\nbody\r\n.\r\n"
        );
    }
}

#[cfg(test)]
#[cfg(all(
    feature = "smtp-transport",
    any(feature = "native-tls", feature = "rustls", feature = "boring-tls")
))]
mod starttls_injection {
    use std::{
        io::{BufRead, BufReader, Write},
        net::{SocketAddr, TcpListener},
        thread,
        time::Duration,
    };

    use lettre::transport::smtp::{
        client::{SmtpConnection, TlsParameters},
        extension::ClientId,
    };

    // Plays a server, or someone in front of it, that appends replies of its
    // own to the reply to STARTTLS, in the same write
    fn server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut commands = BufReader::new(stream.try_clone().unwrap()).lines();
            stream.write_all(b"220 localhost ESMTP\r\n").unwrap();
            let _ehlo = commands.next();
            stream
                .write_all(b"250-localhost\r\n250 STARTTLS\r\n")
                .unwrap();
            let _starttls = commands.next();
            stream
                .write_all(b"220 go ahead\r\n250-forged.example\r\n250 AUTH PLAIN\r\n")
                .unwrap();
        });
        addr
    }

    #[test]
    fn plaintext_after_starttls_reply_is_rejected() {
        let hello = ClientId::Domain("test".into());
        let mut conn =
            SmtpConnection::connect(server(), Some(Duration::from_secs(5)), &hello, None, None)
                .unwrap();
        let tls = TlsParameters::new("localhost".into()).unwrap();

        let err = conn.starttls(&tls, &hello).unwrap_err();
        assert!(err.is_response(), "{err:?}");
        assert!(conn.has_broken());
    }

    #[cfg(feature = "tokio1")]
    mod tokio_1 {
        use lettre::transport::smtp::client::AsyncSmtpConnection;
        use tokio1_crate as tokio;

        use super::*;

        #[tokio::test]
        async fn plaintext_after_starttls_reply_is_rejected() {
            let hello = ClientId::Domain("test".into());
            let mut conn = AsyncSmtpConnection::connect_tokio1(
                server(),
                Some(Duration::from_secs(5)),
                &hello,
                None,
                None,
            )
            .await
            .unwrap();
            let tls = TlsParameters::new("localhost".into()).unwrap();

            let err = conn.starttls(tls, &hello).await.unwrap_err();
            assert!(err.is_response(), "{err:?}");
            assert!(conn.has_broken());
        }
    }
}
