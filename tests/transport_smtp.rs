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
#[cfg(feature = "smtp-transport")]
mod read_response_caps {
    use std::{
        io::{BufRead, BufReader, Write},
        net::{SocketAddr, TcpListener},
        thread,
        time::Duration,
    };

    use lettre::transport::smtp::{Error, client::SmtpConnection, extension::ClientId};

    // Sends `greeting`, answers EHLO if the client gets that far, and keeps
    // the connection open until the client leaves
    fn server(greeting: Vec<u8>) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut commands = BufReader::new(stream.try_clone().unwrap()).lines();
            let _ = stream.write_all(&greeting);
            if commands.next().is_some() {
                let _ = stream.write_all(b"250 localhost\r\n");
            }
            let _ = commands.next();
        });
        addr
    }

    // A reply line longer than the 512 bytes of RFC 5321, as Microsoft 365
    // sends them (#1168)
    fn long_line() -> Vec<u8> {
        format!("220 {}\r\n", "x".repeat(2_000)).into_bytes()
    }

    // More than a reply can hold, with no line ending at all: a client that
    // doesn't bound what it reads buffers all of it, then waits for the rest
    // of the line
    fn unterminated_line() -> Vec<u8> {
        vec![b'x'; 200 * 1024]
    }

    fn connect(addr: SocketAddr) -> Result<SmtpConnection, Error> {
        let hello = ClientId::Domain("test".into());
        SmtpConnection::connect(addr, Some(Duration::from_secs(5)), &hello, None, None)
    }

    #[test]
    fn long_line_is_accepted() {
        connect(server(long_line())).unwrap();
    }

    #[test]
    fn unterminated_line_is_bounded() {
        let err = connect(server(unterminated_line())).err().unwrap();
        assert!(err.is_response(), "{err:?}");
    }

    #[cfg(feature = "tokio1")]
    mod tokio_1 {
        use lettre::transport::smtp::client::AsyncSmtpConnection;
        use tokio1_crate as tokio;

        use super::*;

        // With a time limit of its own: without a bound on what is read,
        // nothing would end the wait for the rest of the line
        async fn connect(addr: SocketAddr) -> Result<AsyncSmtpConnection, Error> {
            let hello = ClientId::Domain("test".into());
            let connect = AsyncSmtpConnection::connect_tokio1(addr, None, &hello, None, None);
            tokio::time::timeout(Duration::from_secs(5), connect)
                .await
                .expect("timed out")
        }

        #[tokio::test]
        async fn long_line_is_accepted() {
            connect(server(long_line())).await.unwrap();
        }

        #[tokio::test]
        async fn unterminated_line_is_bounded() {
            let err = connect(server(unterminated_line())).await.err().unwrap();
            assert!(err.is_response(), "{err:?}");
        }
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

// A reply of the wrong class to DATA, or to the message content, means that
// the replies aren't those to the commands sent: the message must not be sent
// as commands, nor reported as accepted
#[cfg(test)]
#[cfg(feature = "smtp-transport")]
mod reply_class {
    use std::{
        io::{BufRead, BufReader, Write},
        net::{SocketAddr, TcpListener},
        sync::mpsc,
        thread,
        time::Duration,
    };

    use lettre::{
        address::Envelope,
        transport::smtp::{client::SmtpConnection, extension::ClientId},
    };

    const EMAIL: &[u8] = b"Subject: a\r\n\r\nNOOP\r\n";

    // Answers DATA with `data_reply`, and the message content with a `354`.
    // Gives back the lines that the client sent after DATA.
    fn server(data_reply: &'static str) -> (SocketAddr, mpsc::Receiver<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut lines = BufReader::new(stream.try_clone().unwrap()).lines();
            stream.write_all(b"220 localhost ESMTP\r\n").unwrap();
            for _ehlo_mail_rcpt in 0..3 {
                let _command = lines.next();
                stream.write_all(b"250 ok\r\n").unwrap();
            }
            let _data = lines.next();
            stream.write_all(data_reply.as_bytes()).unwrap();

            let mut after_data = Vec::new();
            for line in lines.map_while(Result::ok) {
                if line == "." {
                    let _ = stream.write_all(b"354 go ahead\r\n");
                }
                after_data.push(line);
            }
            let _ = tx.send(after_data);
        });
        (addr, rx)
    }

    fn envelope() -> Envelope {
        Envelope::new(
            Some("from@example.com".parse().unwrap()),
            vec!["to@example.com".parse().unwrap()],
        )
        .unwrap()
    }

    #[test]
    fn completion_reply_to_data_is_rejected() {
        let (addr, after_data) = server("250 ok\r\n");
        let hello = ClientId::Domain("test".into());
        let mut conn =
            SmtpConnection::connect(addr, Some(Duration::from_secs(5)), &hello, None, None)
                .unwrap();

        let err = conn.send(&envelope(), EMAIL).unwrap_err();
        assert!(err.is_response(), "{err:?}");
        assert!(conn.has_broken());
        // Neither the message content, nor QUIT
        assert!(after_data.recv().unwrap().is_empty());
    }

    #[test]
    fn intermediate_reply_to_content_is_rejected() {
        let (addr, _after_data) = server("354 go ahead\r\n");
        let hello = ClientId::Domain("test".into());
        let mut conn =
            SmtpConnection::connect(addr, Some(Duration::from_secs(5)), &hello, None, None)
                .unwrap();

        let err = conn.send(&envelope(), EMAIL).unwrap_err();
        assert!(err.is_response(), "{err:?}");
        assert!(conn.has_broken());
    }

    #[cfg(feature = "tokio1")]
    mod tokio_1 {
        use lettre::transport::smtp::client::AsyncSmtpConnection;
        use tokio1_crate as tokio;

        use super::*;

        async fn connect(addr: SocketAddr) -> AsyncSmtpConnection {
            let hello = ClientId::Domain("test".into());
            AsyncSmtpConnection::connect_tokio1(
                addr,
                Some(Duration::from_secs(5)),
                &hello,
                None,
                None,
            )
            .await
            .unwrap()
        }

        #[tokio::test]
        async fn completion_reply_to_data_is_rejected() {
            let (addr, after_data) = server("250 ok\r\n");
            let mut conn = connect(addr).await;

            let err = conn.send(&envelope(), EMAIL).await.unwrap_err();
            assert!(err.is_response(), "{err:?}");
            assert!(conn.has_broken());
            assert!(after_data.recv().unwrap().is_empty());
        }

        #[tokio::test]
        async fn intermediate_reply_to_content_is_rejected() {
            let (addr, _after_data) = server("354 go ahead\r\n");
            let mut conn = connect(addr).await;

            let err = conn.send(&envelope(), EMAIL).await.unwrap_err();
            assert!(err.is_response(), "{err:?}");
            assert!(conn.has_broken());
        }
    }
}
