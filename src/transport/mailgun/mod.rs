//! This transport uilizes the mailgun API for each email.

use email::SendableEmail;

use transport::EmailTransport;
use transport::mailgun::error::{MailgunResult, Error};

use hyper::header::Headers;

pub mod error;

static BASE_URL : &'static str = "https://api.mailgun.net/v3";

/// Sends an email using the `mailgun` API
#[derive(Debug)]
pub struct MailgunTransport {
    domain: String,
    api_key: String,
}

impl MailgunTransport {
    /// Creates a new transport with the given API Details
    pub fn new(domain: String, api_key: String) -> MailgunTransport {
        MailgunTransport {
            domain: domain,
            api_key: api_key,
        }
    }
}

fn build_headers(transport: &MailgunTransport) -> Headers {
    use hyper::header::{Authorization, Basic, ContentType};
    let mut headers = Headers::new();
    headers.set(
        Authorization(
            Basic {
                username: "api".to_string(),
                password: Some(transport.api_key.clone()),
            }
        )
    );
    headers.set(ContentType({
        use hyper::mime::{Mime, TopLevel, SubLevel};

        Mime(TopLevel::Multipart, SubLevel::FormData, vec![])
    }));

    headers
}



fn build_body<T: SendableEmail>(_transport: &MailgunTransport, email: T) -> MailgunResult<String> {
    use std::collections::HashMap;

    let mut form = HashMap::new();
    form.insert("from", email.from_address());
    form.insert("to", email.to_addresses().join(", "));
    let message = email.message();

    println!("{:#?}", message);
    form.insert("message", message);


    ::serde_urlencoded::to_string(&form).map_err(|e| e.into())
}

impl EmailTransport<MailgunResult<()>> for MailgunTransport {
    fn send<T: SendableEmail>(&mut self, email: T) -> MailgunResult<()> {
        use hyper::status::StatusCode;

        let client = ::hyper::Client::new();
        let headers = build_headers(self);
        let body = try!(build_body(self, email));
        let res = try!(client
            .post(&format!("{base}/{domain}/messages.mime", base = BASE_URL, domain = self.domain))
            .headers(headers)
            .body(&body)
            .send());

        if res.status != StatusCode::Ok {
            Err(Error::Mailgun(res))
        } else {
            Ok(())
        }
    }

    fn close(&mut self) {
        ()
    }
}

