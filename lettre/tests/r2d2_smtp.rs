#[cfg(all(test, feature = "smtp-transport", feature = "connection-pool"))]
mod test {
    extern crate lettre;
    extern crate r2d2;

    use self::lettre::{SmtpConnectionManager, Transport};
    use self::lettre::{ClientSecurity, EmailAddress, Envelope, SendableEmail, SmtpClient};
    use self::r2d2::Pool;
    use std::sync::mpsc;
    use std::thread;

    fn email(message: &str) -> SendableEmail {
        SendableEmail::new(
            Envelope::new(
                Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
                vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
            ).unwrap(),
            "id".to_string(),
            message.to_string().into_bytes(),
        )
    }

    #[test]
    fn send_one() {
        let client = SmtpClient::new("localhost:2525", ClientSecurity::None).unwrap();
        let manager = SmtpConnectionManager::new(client).unwrap();
        let pool = Pool::builder().max_size(1).build(manager).unwrap();

        let mut mailer = pool.get().unwrap();
        let result = (*mailer).send(email("send one"));
        assert!(result.is_ok());
    }

    #[test]
    fn send_from_thread() {
        let client = SmtpClient::new("127.0.0.1:2525", ClientSecurity::None).unwrap();
        let manager = SmtpConnectionManager::new(client).unwrap();
        let pool = Pool::builder().max_size(2).build(manager).unwrap();

        let (s1, r1) = mpsc::channel();
        let (s2, r2) = mpsc::channel();

        let pool1 = pool.clone();
        let t1 = thread::spawn(move || {
            let mut conn = pool1.get().unwrap();
            s1.send(()).unwrap();
            r2.recv().unwrap();
            (*conn)
                .send(email("send from thread 1"))
                .expect("Send failed from thread 1");
            drop(conn);
        });

        let pool2 = pool.clone();
        let t2 = thread::spawn(move || {
            let mut conn = pool2.get().unwrap();
            s2.send(()).unwrap();
            r1.recv().unwrap();
            (*conn)
                .send(email("send from thread 2"))
                .expect("Send failed from thread 2");
            drop(conn);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let mut mailer = pool.get().unwrap();
        (*mailer)
            .send(email("send from main thread"))
            .expect("Send failed from main thread");
    }
}
