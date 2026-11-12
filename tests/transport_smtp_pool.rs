#[cfg(all(test, feature = "smtp-transport", feature = "pool"))]
mod sync {
    use std::{sync::mpsc, thread};

    use lettre::{SmtpTransport, Transport, address::Envelope};

    fn envelope() -> Envelope {
        Envelope::new(
            Some("user@localhost".parse().unwrap()),
            vec!["root@localhost".parse().unwrap()],
        )
        .unwrap()
    }

    #[test]
    fn send_one() {
        let mailer = SmtpTransport::builder_dangerous("127.0.0.1")
            .port(2525)
            .build();

        let result = mailer.send_raw(&envelope(), b"test");
        assert!(result.is_ok());
    }

    #[test]
    fn send_from_thread() {
        let mailer = SmtpTransport::builder_dangerous("127.0.0.1")
            .port(2525)
            .build();

        let (s1, r1) = mpsc::channel();
        let (s2, r2) = mpsc::channel();

        let mailer1 = mailer.clone();
        let t1 = thread::spawn(move || {
            s1.send(()).unwrap();
            r2.recv().unwrap();
            mailer1
                .send_raw(&envelope(), b"test1")
                .expect("Send failed from thread 1");
        });

        let mailer2 = mailer.clone();
        let t2 = thread::spawn(move || {
            s2.send(()).unwrap();
            r1.recv().unwrap();
            mailer2
                .send_raw(&envelope(), b"test2")
                .expect("Send failed from thread 2");
        });

        t1.join().unwrap();
        t2.join().unwrap();

        mailer
            .send_raw(&envelope(), b"test")
            .expect("Send failed from main thread");
    }
}

#[cfg(all(test, feature = "smtp-transport", feature = "pool", feature = "tokio1"))]
mod tokio_1 {
    use std::{
        io::{BufRead, BufReader, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
        time::Duration,
    };

    use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor, address::Envelope};
    use tokio1_crate as tokio;

    fn envelope() -> Envelope {
        Envelope::new(
            Some("user@localhost".parse().unwrap()),
            vec!["root@localhost".parse().unwrap()],
        )
        .unwrap()
    }

    /// How the first connection misbehaves.
    #[derive(Clone, Copy)]
    enum Stall {
        /// Answer `DATA` only after this long.
        Reply(Duration),
        /// Stop reading for this long after answering `DATA`.
        Read(Duration),
    }

    /// A minimal SMTP server. Its first connection stalls as given and reports when the
    /// client closes it; every connection treats lines as message content until `.`.
    fn server(stall: Stall) -> (u16, mpsc::Receiver<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (closed, first_closed) = mpsc::channel();
        thread::spawn(move || {
            for (n, stream) in listener.incoming().enumerate() {
                let Ok(stream) = stream else { return };
                let stall = (n == 0).then_some(stall);
                let closed = closed.clone();
                thread::spawn(move || {
                    serve(stream, stall);
                    if n == 0 {
                        let _ = closed.send(());
                    }
                });
            }
        });
        (port, first_closed)
    }

    /// Waits without blocking the runtime, so the pool can recycle connections meanwhile.
    async fn closed_within(closed: mpsc::Receiver<()>, timeout: Duration) -> bool {
        tokio::task::spawn_blocking(move || closed.recv_timeout(timeout).is_ok())
            .await
            .unwrap()
    }

    fn serve(stream: TcpStream, stall: Option<Stall>) {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = stream;
        let _ = writer.write_all(b"220 localhost ESMTP\r\n");
        let mut in_data = false;
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                return;
            }
            if in_data {
                if line == ".\r\n" {
                    in_data = false;
                    let _ = writer.write_all(b"250 queued\r\n");
                }
                continue;
            }
            let reply: &[u8] = match line.get(..4).map(str::to_ascii_uppercase).as_deref() {
                Some("DATA") => {
                    if let Some(Stall::Reply(delay)) = stall {
                        thread::sleep(delay);
                    }
                    in_data = true;
                    let _ = writer.write_all(b"354 go ahead\r\n");
                    if let Some(Stall::Read(delay)) = stall {
                        thread::sleep(delay);
                    }
                    continue;
                }
                Some("QUIT") => {
                    let _ = writer.write_all(b"221 bye\r\n");
                    return;
                }
                _ => b"250 ok\r\n",
            };
            let _ = writer.write_all(reply);
        }
    }

    #[tokio::test]
    async fn send_after_cancelled_send() {
        let (port, first_closed) = server(Stall::Reply(Duration::from_millis(200)));
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("127.0.0.1")
                .port(port)
                .build();

        // Dropped while waiting for the reply to DATA.
        let cancelled = tokio::time::timeout(
            Duration::from_millis(100),
            mailer.send_raw(&envelope(), b"first"),
        )
        .await;
        assert!(
            cancelled.is_err(),
            "the first send should have been cancelled"
        );

        // Let the late 354 arrive and the dropped connection go back to the pool.
        tokio::time::sleep(Duration::from_millis(300)).await;

        let sent = tokio::time::timeout(
            Duration::from_secs(5),
            mailer.send_raw(&envelope(), b"second"),
        )
        .await;
        assert!(
            matches!(sent, Ok(Ok(_))),
            "send after a cancelled send: {sent:?}"
        );

        // The abandoned connection is closed rather than left waiting for a reply to QUIT.
        assert!(
            closed_within(first_closed, Duration::from_secs(5)).await,
            "the connection of the cancelled send was not closed"
        );
    }

    #[tokio::test]
    async fn connection_of_cancelled_message_is_closed() {
        let (port, first_closed) = server(Stall::Read(Duration::from_millis(500)));
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("127.0.0.1")
                .port(port)
                .build();

        // Dropped while writing a message the server is not reading yet.
        let message = vec![b'x'; 16 << 20];
        let cancelled = tokio::time::timeout(
            Duration::from_millis(200),
            mailer.send_raw(&envelope(), &message),
        )
        .await;
        assert!(cancelled.is_err(), "the send should have been cancelled");

        // The server is in the middle of the message, so a QUIT would get no reply: the
        // connection must be closed rather than left waiting for one.
        assert!(
            closed_within(first_closed, Duration::from_secs(5)).await,
            "the connection of the cancelled send was not closed"
        );

        let sent = tokio::time::timeout(
            Duration::from_secs(5),
            mailer.send_raw(&envelope(), b"second"),
        )
        .await;
        assert!(
            matches!(sent, Ok(Ok(_))),
            "send after a cancelled send: {sent:?}"
        );
    }
}
