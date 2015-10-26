//! This transport is a stub that only logs the message, and always returns
//! succes

use transport::error::EmailResult;
use transport::smtp::response::{Category, Code, Response, Severity};
use transport::EmailTransport;
use email::SendableEmail;

/// This transport does nothing exept logging the message enveloppe
pub struct StubEmailTransport;

impl EmailTransport for StubEmailTransport {
    fn send<T: SendableEmail>(&mut self, email: T) -> EmailResult {

        info!("{}: from=<{}> to=<{:?}>",
              email.message_id(),
              email.from_address(),
              email.to_addresses());
        Ok(Response::new(Code::new(Severity::PositiveCompletion, Category::MailSystem, 0),
                         vec!["Ok: email logged".to_string()]))
    }

    fn close(&mut self) {
        ()
    }
}
