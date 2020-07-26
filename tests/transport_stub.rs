use lettre::{transport::stub::StubTransport, Message};
use tokio02_crate as tokio;

#[test]
fn stub_transport() {
    use lettre::Transport;
    let sender_ok = StubTransport::new_ok();
    let sender_ko = StubTransport::new_error();
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body("Be happy!")
        .unwrap();

    sender_ok.send(&email).unwrap();
    sender_ko.send(&email).unwrap_err();
}

#[cfg(feature = "async-std1")]
#[async_attributes::test]
async fn stub_transport_asyncstd1() {
    use lettre::AsyncStd1Transport;

    let sender_ok = StubTransport::new_ok();
    let sender_ko = StubTransport::new_error();
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
        .body("Be happy!")
        .unwrap();

    sender_ok.send(email.clone()).await.unwrap();
    sender_ko.send(email).await.unwrap_err();
}

#[cfg(feature = "tokio02")]
#[tokio::test]
async fn stub_transport_tokio02() {
    use lettre::Tokio02Transport;

    let sender_ok = StubTransport::new_ok();
    let sender_ko = StubTransport::new_error();
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
        .body("Be happy!")
        .unwrap();

    sender_ok.send(email.clone()).await.unwrap();
    sender_ko.send(email).await.unwrap_err();
}
