extern crate lettre;

use lettre::{EmailAddress, EmailTransport, SimpleSendableEmail};
use lettre::smtp::response::{Code, Response};
use lettre::stub::StubEmailTransport;
use std::str::FromStr;

#[test]
fn stub_transport() {
    let mut sender_ok = StubEmailTransport::new_positive();
    let response_ok = Response::new(Code::from_str("200").unwrap(), vec!["ok".to_string()]);
    let response_ko = Response::new(Code::from_str("510").unwrap(), vec!["ko".to_string()]);
    let mut sender_ko = StubEmailTransport::new(response_ko);

    let email = SimpleSendableEmail::new(
        EmailAddress::new("user@localhost".to_string()),
        vec![EmailAddress::new("root@localhost".to_string())],
        "stub_id".to_string(),
        "Hello stub".to_string(),
    );

    let result_ok = sender_ok.send(&email).unwrap();
    let result_ko = sender_ko.send(&email);

    assert_eq!(result_ok, response_ok);
    assert!(result_ko.is_err());

}
