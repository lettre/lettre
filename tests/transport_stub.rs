#[cfg(test)]
#[cfg(feature = "builder")]
mod sync {
    use lettre::{transport::stub::StubTransport, Message, Transport};

    #[test]
    fn stub_transport() {
        let sender_ok = StubTransport::new_ok();
        let sender_ko = StubTransport::new_error();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .body(String::from("Be happy!"))
            .unwrap();

        sender_ok.send(&email).unwrap();
        sender_ko.send(&email).unwrap_err();
    }
}

#[cfg(test)]
#[cfg(all(feature = "builder", feature = "tokio02"))]
mod tokio_02 {
    use lettre::{transport::stub::StubTransport, AsyncTransport, Message};

    use tokio02_crate as tokio;

    #[tokio::test]
    async fn stub_transport_tokio02() {
        let sender_ok = StubTransport::new_ok();
        let sender_ko = StubTransport::new_error();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
            .body(String::from("Be happy!"))
            .unwrap();

        sender_ok.send(email.clone()).await.unwrap();
        sender_ko.send(email).await.unwrap_err();
    }
}

#[cfg(test)]
#[cfg(all(feature = "builder", feature = "tokio1"))]
mod tokio_1 {
    use lettre::{transport::stub::StubTransport, AsyncTransport, Message};

    use tokio1_crate as tokio;

    #[tokio::test]
    async fn stub_transport_tokio1() {
        let sender_ok = StubTransport::new_ok();
        let sender_ko = StubTransport::new_error();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
            .body(String::from("Be happy!"))
            .unwrap();

        sender_ok.send(email.clone()).await.unwrap();
        sender_ko.send(email).await.unwrap_err();
    }
}

#[cfg(test)]
#[cfg(all(feature = "builder", feature = "async-std1"))]
mod asyncstd_1 {
    use lettre::{transport::stub::StubTransport, AsyncTransport, Message};

    #[async_std::test]
    async fn stub_transport_asyncstd1() {
        let sender_ok = StubTransport::new_ok();
        let sender_ko = StubTransport::new_error();
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
            .body(String::from("Be happy!"))
            .unwrap();

        sender_ok.send(email.clone()).await.unwrap();
        sender_ko.send(email).await.unwrap_err();
    }
}
