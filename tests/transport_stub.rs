use lettre::{transport::stub::StubTransport, Message, Transport};

#[test]
fn stub_transport() {
    let sender_ok = StubTransport::new_positive();
    let sender_ko = StubTransport::new(Err(()));
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body("Be happy!")
        .unwrap();

    sender_ok.send(&email.clone()).unwrap();
    sender_ko.send(&email).unwrap_err();
}
