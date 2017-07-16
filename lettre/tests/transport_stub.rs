extern crate lettre;

use lettre::{EmailTransport, SimpleSendableEmail};
use lettre::stub::StubEmailTransport;
use lettre::smtp::response::{Code, Response};
use std::str::FromStr;

#[test]
fn stub_transport() {
    let mut sender_ok = StubEmailTransport::new_positive();
    let response_ok = Response::new(Code::from_str("200").unwrap(), vec!["ok".to_string()]);
    let response_ko = Response::new(Code::from_str("510").unwrap(), vec!["ko".to_string()]);
    let mut sender_ko = StubEmailTransport::new(response_ko);

    let email = SimpleSendableEmail::new(
        "user@localhost",
        vec!["root@localhost"],
        "stub_id",
        "Hello stub",
    );

    let result_ok = sender_ok.send(email.clone()).unwrap();
    let result_ko = sender_ko.send(email);

    assert_eq!(result_ok, response_ok);
    assert!(result_ko.is_err());

}
