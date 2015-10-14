//! TODO

use transport::error::EmailResult;
use transport::smtp::response::Response;
use transport::EmailTransport;
use transport::smtp::response::{Code, Category, Severity};

/// TODO
pub struct StubEmailTransport;

impl EmailTransport for StubEmailTransport {
    fn send(&mut self,
            to_addresses: Vec<String>,
            from_address: String,
            message: String,
            message_id: String)
            -> EmailResult {

        let _ = message;
        info!("message '{}': from '{}' to '{:?}'",
              message_id,
              from_address,
              to_addresses);
        Ok(Response::new(Code::new(Severity::PositiveCompletion, Category::MailSystem, 0),
                         vec!["Ok: email logged".to_string()]))
    }

    fn close(&mut self) {
        ()
    }
}
