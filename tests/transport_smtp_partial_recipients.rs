//! Integration tests for the opt-in partial recipient handling of the SMTP
//! transport.
//!
//! A scripted fake SMTP server bound to `127.0.0.1:0` keeps the tests
//! self-contained. It replies `550` to any `RCPT` whose address contains
//! `bad`, accepts the others, and records a transcript of the whole
//! conversation so the tests can assert on the wire behaviour.

#![cfg(all(feature = "smtp-transport", feature = "builder"))]

use std::{
    io::{BufRead, BufReader, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    time::Duration,
};

use lettre::Message;

/// How the fake server should behave.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Normal SMTP conversation.
    Normal,
    /// Close the socket abruptly right after replying to `MAIL FROM`.
    CloseAfterMail,
}

struct Server {
    port: u16,
    log: Receiver<String>,
    #[cfg_attr(not(feature = "pool"), allow(dead_code))]
    accepts: Arc<AtomicUsize>,
}

impl Server {
    /// Everything sent (`S:`) and received (`C:`) so far.
    fn transcript(&self) -> Vec<String> {
        self.log.try_iter().collect()
    }
}

fn send_line(w: &mut TcpStream, tx: &Sender<String>, line: &str) {
    // Record before writing: the client can only observe the reply after the
    // flush, so the transcript entry is always queued first.
    let _ = tx.send(format!("S: {}", line.trim_end()));
    let _ = w.write_all(line.as_bytes());
    let _ = w.flush();
}

fn start_server(mode: Mode) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, log) = channel::<String>();
    let accepts = Arc::new(AtomicUsize::new(0));
    let accepts_ = Arc::clone(&accepts);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(_) => break,
            };
            accepts_.fetch_add(1, Ordering::SeqCst);
            let tx = tx.clone();
            std::thread::spawn(move || handle_conn(stream, mode, tx));
        }
    });

    Server { port, log, accepts }
}

fn handle_conn(stream: TcpStream, mode: Mode, tx: Sender<String>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let mut w = stream.try_clone().unwrap();
    let mut r = BufReader::new(stream);

    send_line(&mut w, &tx, "220 fake ESMTP\r\n");

    let mut accepted = 0usize;
    let mut line = String::new();
    loop {
        line.clear();
        match r.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        let cmd = line.trim_end().to_owned();
        let _ = tx.send(format!("C: {cmd}"));
        let upper = cmd.to_ascii_uppercase();

        if upper.starts_with("EHLO") {
            send_line(&mut w, &tx, "250-fake greets you\r\n");
            send_line(&mut w, &tx, "250-SIZE 10485760\r\n");
            send_line(&mut w, &tx, "250-8BITMIME\r\n");
            send_line(&mut w, &tx, "250 HELP\r\n");
        } else if upper.starts_with("MAIL") {
            send_line(&mut w, &tx, "250 2.1.0 Ok\r\n");
            if mode == Mode::CloseAfterMail {
                let _ = tx.send("S: [abrupt close after MAIL reply]".to_owned());
                let _ = w.shutdown(Shutdown::Both);
                return;
            }
        } else if upper.starts_with("RCPT") {
            if cmd.contains("bad") {
                let address = cmd
                    .split('<')
                    .nth(1)
                    .and_then(|rest| rest.split('>').next())
                    .unwrap_or("unknown")
                    .to_owned();
                send_line(
                    &mut w,
                    &tx,
                    &format!("550 5.1.1 <{address}> Recipient address rejected: User unknown\r\n"),
                );
            } else {
                accepted += 1;
                send_line(&mut w, &tx, "250 2.1.5 Ok\r\n");
            }
        } else if upper.starts_with("DATA") {
            if accepted >= 1 {
                send_line(&mut w, &tx, "354 End data with <CR><LF>.<CR><LF>\r\n");
                loop {
                    let mut body = String::new();
                    match r.read_line(&mut body) {
                        Ok(0) | Err(_) => break,
                        Ok(_) if body == ".\r\n" => {
                            let _ = tx.send("C: [message body; . terminator]".to_owned());
                            break;
                        }
                        Ok(_) => {}
                    }
                }
                send_line(&mut w, &tx, "250 2.0.0 Ok: queued as 42\r\n");
            } else {
                send_line(&mut w, &tx, "554 5.5.1 No valid recipients\r\n");
            }
        } else if upper.starts_with("NOOP") {
            send_line(&mut w, &tx, "250 2.0.0 Ok\r\n");
        } else if upper.starts_with("RSET") {
            accepted = 0;
            send_line(&mut w, &tx, "250 2.0.0 Ok\r\n");
        } else if upper.starts_with("QUIT") {
            send_line(&mut w, &tx, "221 2.0.0 Bye\r\n");
            let _ = w.shutdown(Shutdown::Both);
            return;
        } else {
            send_line(&mut w, &tx, "250 2.0.0 Ok\r\n");
        }
    }
}

/// Message with one acceptable and one rejected recipient.
fn mixed_message() -> Message {
    Message::builder()
        .from("Sender <sender@example.com>".parse().unwrap())
        .to("Ok <ok@example.com>".parse().unwrap())
        .cc("Bad <bad@example.com>".parse().unwrap())
        .subject("Partial recipients")
        .body(String::from("Hello"))
        .unwrap()
}

/// Message whose only recipients are all rejected by the server.
fn all_bad_message() -> Message {
    Message::builder()
        .from("Sender <sender@example.com>".parse().unwrap())
        .to("Bad <bad@example.com>".parse().unwrap())
        .subject("All bad recipients")
        .body(String::from("Hello"))
        .unwrap()
}

fn data_attempted(transcript: &[String]) -> bool {
    transcript.iter().any(|line| line.starts_with("C: DATA"))
}

#[cfg(all(feature = "smtp-transport", feature = "builder"))]
mod sync {
    use lettre::{SmtpTransport, Transport};

    use super::*;

    fn transport(server: &Server) -> SmtpTransport {
        SmtpTransport::builder_dangerous("127.0.0.1")
            .port(server.port)
            .timeout(Some(Duration::from_secs(10)))
            .build()
    }

    fn partial_transport(server: &Server) -> SmtpTransport {
        SmtpTransport::builder_dangerous("127.0.0.1")
            .port(server.port)
            .timeout(Some(Duration::from_secs(10)))
            .allow_partial_recipients(true)
            .build()
    }

    #[test]
    fn partial_recipients_delivers_to_accepted_recipients() {
        let server = start_server(Mode::Normal);

        let result = partial_transport(&server).send(&mixed_message());
        let log = server.transcript();

        assert!(result.is_ok(), "send should succeed, got {result:?}");
        assert!(
            data_attempted(&log),
            "the message must still be DATA'd: {log:#?}"
        );
        assert!(
            log.contains(&"S: 250 2.0.0 Ok: queued as 42".to_owned()),
            "the message must be accepted: {log:#?}"
        );
        assert!(
            log.contains(&"S: 250 2.1.5 Ok".to_owned()),
            "the accepted recipient must get a 250: {log:#?}"
        );
        assert!(
            log.contains(
                &"S: 550 5.1.1 <bad@example.com> Recipient address rejected: User unknown"
                    .to_owned()
            ),
            "the rejected recipient must get a 550: {log:#?}"
        );
    }

    #[test]
    fn no_accepted_recipient_fails_without_data() {
        let server = start_server(Mode::Normal);

        let result = partial_transport(&server).send(&all_bad_message());
        let log = server.transcript();

        assert!(result.is_err(), "send should fail, got {result:?}");
        assert!(
            !data_attempted(&log),
            "no message data may be sent without an accepted recipient: {log:#?}"
        );
    }

    #[test]
    fn rejected_recipient_fails_the_send_by_default() {
        let server = start_server(Mode::Normal);

        let result = transport(&server).send(&mixed_message());
        let log = server.transcript();

        assert!(result.is_err(), "send should fail, got {result:?}");
        assert!(
            !data_attempted(&log),
            "no message data may be sent once a recipient is rejected: {log:#?}"
        );
    }

    #[test]
    fn network_error_stays_fatal() {
        let server = start_server(Mode::CloseAfterMail);

        let result = partial_transport(&server).send(&mixed_message());
        let log = server.transcript();

        assert!(result.is_err(), "send should fail, got {result:?}");
        assert!(
            !data_attempted(&log),
            "no message data may be sent after a transport error: {log:#?}"
        );
    }

    #[cfg(feature = "pool")]
    #[test]
    fn partial_recipients_reuse_pooled_connection() {
        use lettre::transport::smtp::PoolConfig;

        let server = start_server(Mode::Normal);
        let transport = SmtpTransport::builder_dangerous("127.0.0.1")
            .port(server.port)
            .timeout(Some(Duration::from_secs(10)))
            .allow_partial_recipients(true)
            .pool_config(PoolConfig::new().max_size(1))
            .build();

        transport.send(&mixed_message()).unwrap();
        transport.send(&mixed_message()).unwrap();

        assert_eq!(
            server.accepts.load(Ordering::SeqCst),
            1,
            "the second send should reuse the pooled connection"
        );
    }
}

#[cfg(all(feature = "smtp-transport", feature = "builder", feature = "tokio1"))]
mod tokio1 {
    use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
    use tokio1_crate as tokio;

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn partial_recipients_delivers_to_accepted_recipients() {
        let server = start_server(Mode::Normal);
        let transport: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("127.0.0.1")
                .port(server.port)
                .timeout(Some(Duration::from_secs(10)))
                .allow_partial_recipients(true)
                .build();

        let result = tokio::time::timeout(Duration::from_secs(30), transport.send(mixed_message()))
            .await
            .expect("send must not hang");
        let log = server.transcript();

        assert!(result.is_ok(), "send should succeed, got {result:?}");
        assert!(
            data_attempted(&log),
            "the message must still be DATA'd: {log:#?}"
        );
        assert!(
            log.contains(&"S: 250 2.0.0 Ok: queued as 42".to_owned()),
            "the message must be accepted: {log:#?}"
        );
        assert!(
            log.contains(
                &"S: 550 5.1.1 <bad@example.com> Recipient address rejected: User unknown"
                    .to_owned()
            ),
            "the rejected recipient must get a 550: {log:#?}"
        );
    }
}
