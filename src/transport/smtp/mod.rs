//! Sends an email using the client

use std::string::String;
use std::net::{SocketAddr, ToSocketAddrs};

use openssl::ssl::{SslContext, SslMethod};

use transport::error::{EmailResult, Error};
use transport::smtp::extension::{Extension, ServerInfo};
use transport::smtp::client::Client;
use transport::smtp::authentication::Mechanism;
use transport::EmailTransport;
use email::SendableEmail;

pub mod extension;
pub mod authentication;
pub mod response;
pub mod client;

// Registrated port numbers:
// https://www.iana.
// org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml

/// Default smtp port
pub const SMTP_PORT: u16 = 25;

/// Default submission port
pub const SUBMISSION_PORT: u16 = 587;

// Useful strings and characters

/// The word separator for SMTP transactions
pub const SP: &'static str = " ";

/// The line ending for SMTP transactions (carriage return + line feed)
pub const CRLF: &'static str = "\r\n";

/// Colon
pub const COLON: &'static str = ":";

/// The ending of message content
pub const MESSAGE_ENDING: &'static str = "\r\n.\r\n";

/// NUL unicode character
pub const NUL: &'static str = "\0";

/// TLS security level
#[derive(Debug)]
pub enum SecurityLevel {
    /// Use a TLS wrapped connection
    ///
    /// Non RFC-compliant, should only be used if the server does not support STARTTLS.
    EncryptedWrapper,
    /// Only send an email on encrypted connection (with STARTTLS)
    ///
    /// Recommended mode, prevents MITM when used with verified certificates.
    AlwaysEncrypt,
    /// Use TLS when available (with STARTTLS)
    ///
    /// Default mode.
    Opportunistic,
    /// Never use TLS
    NeverEncrypt,
}

/// Contains client configuration
pub struct SmtpTransportBuilder {
    /// Maximum connection reuse
    ///
    /// Zero means no limitation
    connection_reuse_count_limit: u16,
    /// Enable connection reuse
    connection_reuse: bool,
    /// Name sent during HELO or EHLO
    hello_name: String,
    /// Credentials
    credentials: Option<(String, String)>,
    /// Socket we are connecting to
    server_addr: SocketAddr,
    /// SSL context to use
    ssl_context: SslContext,
    /// TLS security level
    security_level: SecurityLevel,
    /// Enable UTF8 mailboxes in envelope or headers
    smtp_utf8: bool,
    /// List of authentication mechanisms, sorted by priority
    authentication_mechanisms: Vec<Mechanism>,
}

/// Builder for the SMTP SmtpTransport
impl SmtpTransportBuilder {
    /// Creates a new local SMTP client
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<SmtpTransportBuilder, Error> {
        let mut addresses = try!(addr.to_socket_addrs());

        match addresses.next() {
            Some(addr) => {
                Ok(SmtpTransportBuilder {
                    server_addr: addr,
                    ssl_context: SslContext::new(SslMethod::Tlsv1).unwrap(),
                    security_level: SecurityLevel::Opportunistic,
                    smtp_utf8: false,
                    credentials: None,
                    connection_reuse_count_limit: 100,
                    connection_reuse: false,
                    hello_name: "localhost".to_string(),
                    authentication_mechanisms: vec![Mechanism::CramMd5, Mechanism::Plain],
                })
            }
            None => Err(From::from("Could nor resolve hostname")),
        }
    }

    /// Creates a new local SMTP client to port 25
    pub fn localhost() -> Result<SmtpTransportBuilder, Error> {
        SmtpTransportBuilder::new(("localhost", SMTP_PORT))
    }

    /// Use STARTTLS with a specific context
    pub fn ssl_context(mut self, ssl_context: SslContext) -> SmtpTransportBuilder {
        self.ssl_context = ssl_context;
        self
    }

    /// Set the security level for SSL/TLS
    pub fn security_level(mut self, level: SecurityLevel) -> SmtpTransportBuilder {
        self.security_level = level;
        self
    }

    /// Require SSL/TLS using STARTTLS
    ///
    /// Incompatible with `ssl_wrapper()``
    pub fn encrypt(mut self) -> SmtpTransportBuilder {
        self.security_level = SecurityLevel::AlwaysEncrypt;
        self
    }

    /// Require SSL/TLS using STARTTLS
    ///
    /// Incompatible with `encrypt()`
    pub fn ssl_wrapper(mut self) -> SmtpTransportBuilder {
        self.security_level = SecurityLevel::EncryptedWrapper;
        self
    }

    /// Enable SMTPUTF8 if the server supports it
    pub fn smtp_utf8(mut self, enabled: bool) -> SmtpTransportBuilder {
        self.smtp_utf8 = enabled;
        self
    }

    /// Set the name used during HELO or EHLO
    pub fn hello_name(mut self, name: &str) -> SmtpTransportBuilder {
        self.hello_name = name.to_string();
        self
    }

    /// Enable connection reuse
    pub fn connection_reuse(mut self, enable: bool) -> SmtpTransportBuilder {
        self.connection_reuse = enable;
        self
    }

    /// Set the maximum number of emails sent using one connection
    pub fn connection_reuse_count_limit(mut self, limit: u16) -> SmtpTransportBuilder {
        self.connection_reuse_count_limit = limit;
        self
    }

    /// Set the client credentials
    pub fn credentials(mut self, username: &str, password: &str) -> SmtpTransportBuilder {
        self.credentials = Some((username.to_string(), password.to_string()));
        self
    }

    /// Set the authentication mechanisms
    pub fn authentication_mechanisms(mut self, mechanisms: Vec<Mechanism>) -> SmtpTransportBuilder {
        self.authentication_mechanisms = mechanisms;
        self
    }

    /// Build the SMTP client
    ///
    /// It does not connects to the server, but only creates the `SmtpTransport`
    pub fn build(self) -> SmtpTransport {
        SmtpTransport::new(self)
    }
}

/// Represents the state of a client
#[derive(Debug)]
struct State {
    /// Panic state
    pub panic: bool,
    /// Connection reuse counter
    pub connection_reuse_count: u16,
}

/// Structure that implements the high level SMTP client
pub struct SmtpTransport {
    /// Information about the server
    /// Value is None before HELO/EHLO
    server_info: Option<ServerInfo>,
    /// SmtpTransport variable states
    state: State,
    /// Information about the client
    client_info: SmtpTransportBuilder,
    /// Low level client
    client: Client,
}

macro_rules! try_smtp (
    ($err: expr, $client: ident) => ({
        match $err {
            Ok(val) => val,
            Err(err) => {
                if !$client.state.panic {
                    $client.state.panic = true;
                    $client.reset();
                }
                return Err(From::from(err))
            },
        }
    })
);

impl SmtpTransport {
    /// Creates a new SMTP client
    ///
    /// It does not connect to the server, but only creates the `SmtpTransport`
    pub fn new(builder: SmtpTransportBuilder) -> SmtpTransport {

        let client = Client::new();

        SmtpTransport {
            client: client,
            server_info: None,
            client_info: builder,
            state: State {
                panic: false,
                connection_reuse_count: 0,
            },
        }
    }

    /// Reset the client state
    fn reset(&mut self) {
        // Close the SMTP transaction if needed
        self.close();

        // Reset the client state
        self.server_info = None;
        self.state.panic = false;
        self.state.connection_reuse_count = 0;
    }

    /// Gets the EHLO response and updates server information
    pub fn get_ehlo(&mut self) -> EmailResult {
        // Extended Hello
        let ehlo_response = try_smtp!(self.client.ehlo(&self.client_info.hello_name), self);

        self.server_info = Some(try_smtp!(ServerInfo::from_response(&ehlo_response), self));

        // Print server information
        debug!("server {}", self.server_info.as_ref().unwrap());

        Ok(ehlo_response)
    }
}

impl EmailTransport for SmtpTransport {
    /// Sends an email
    fn send<T: SendableEmail>(&mut self, email: T) -> EmailResult {

        // Extract email information
        let message_id = email.message_id();
        let from_address = email.from_address();
        let to_addresses = email.to_addresses();
        let message = email.message();

        // Check if the connection is still available
        if self.state.connection_reuse_count > 0 {
            if !self.client.is_connected() {
                self.reset();
            }
        }

        if self.state.connection_reuse_count == 0 {
            try!(self.client.connect(&self.client_info.server_addr,
                                     match &self.client_info.security_level {
                                         &SecurityLevel::EncryptedWrapper => {
                                             Some(&self.client_info.ssl_context)
                                         }
                                         _ => None,
                                     }));

            // Log the connection
            info!("connection established to {}", self.client_info.server_addr);

            try!(self.get_ehlo());

            match (&self.client_info.security_level,
                   self.server_info.as_ref().unwrap().supports_feature(&Extension::StartTls)) {
                (&SecurityLevel::AlwaysEncrypt, false) => {
                    return Err(From::from("Could not encrypt connection, aborting"))
                }
                (&SecurityLevel::Opportunistic, false) => (),
                (&SecurityLevel::NeverEncrypt, _) => (),
                (&SecurityLevel::EncryptedWrapper, _) => (),
                (_, true) => {
                    try_smtp!(self.client.starttls(), self);
                    try_smtp!(self.client.upgrade_tls_stream(&self.client_info.ssl_context),
                              self);

                    debug!("connection encrypted");

                    // Send EHLO again
                    try!(self.get_ehlo());
                }
            }

            if self.client_info.credentials.is_some() {
                let (username, password) = self.client_info.credentials.clone().unwrap();

                let mut found = false;

                for mechanism in self.client_info.authentication_mechanisms.clone() {
                    if self.server_info.as_ref().unwrap().supports_auth_mechanism(mechanism) {
                        found = true;
                        try_smtp!(self.client.auth(mechanism, &username, &password), self);
                        break;
                    }
                }

                if !found {
                    info!("No supported authentication mechanisms available");
                }
            }
        }

        // Mail
        let mail_options = match (self.server_info
                                      .as_ref()
                                      .unwrap()
                                      .supports_feature(&Extension::EightBitMime),
                                  self.server_info
                                      .as_ref()
                                      .unwrap()
                                      .supports_feature(&Extension::SmtpUtfEight)) {
            (true, true) => Some("BODY=8BITMIME SMTPUTF8"),
            (true, false) => Some("BODY=8BITMIME"),
            (false, _) => None,
        };

        try_smtp!(self.client.mail(&from_address, mail_options), self);

        // Log the mail command
        info!("{}: from=<{}>", message_id, from_address);

        // Recipient
        for to_address in to_addresses.iter() {
            try_smtp!(self.client.rcpt(&to_address), self);
            // Log the rcpt command
            info!("{}: to=<{}>", message_id, to_address);
        }

        // Data
        try_smtp!(self.client.data(), self);

        // Message content
        let result = self.client.message(&message);

        if result.is_ok() {
            // Increment the connection reuse counter
            self.state.connection_reuse_count = self.state.connection_reuse_count + 1;

            // Log the message
            info!("{}: conn_use={}, size={}, status=sent ({})",
                  message_id,
                  self.state.connection_reuse_count,
                  message.len(),
                  result.as_ref()
                        .ok()
                        .unwrap()
                        .message()
                        .iter()
                        .next()
                        .unwrap_or(&"no response".to_string()));
        }

        // Test if we can reuse the existing connection
        if (!self.client_info.connection_reuse) ||
           (self.state.connection_reuse_count >= self.client_info.connection_reuse_count_limit) {
            self.reset();
        }

        result
    }

    /// Closes the inner connection
    fn close(&mut self) {
        self.client.close();
    }
}
