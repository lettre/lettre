#[cfg(all(test, feature = "smtp-transport", feature = "r2d2"))]
mod sync {
    use lettre::{address::Envelope, SmtpTransport, Transport};
    use std::{sync::mpsc, thread};

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
