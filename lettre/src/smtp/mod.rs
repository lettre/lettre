//! The SMTP transport sends emails using the SMTP protocol.
//!
//! This SMTP client follows [RFC
//! 5321](https://tools.ietf.org/html/rfc5321), and is designed to efficiently send emails from an
//! application to a relay email server, as it relies as much as possible on the relay server
//! for sanity and RFC compliance checks.
//!
//! It implements the following extensions:
//!
//! * 8BITMIME ([RFC 6152](https://tools.ietf.org/html/rfc6152))
//! * AUTH ([RFC 4954](http://tools.ietf.org/html/rfc4954)) with PLAIN, LOGIN and
//! CRAM-MD5 mechanisms
//! * STARTTLS ([RFC 2487](http://tools.ietf.org/html/rfc2487))
//! * SMTPUTF8 ([RFC 6531](http://tools.ietf.org/html/rfc6531))
//!

use EmailTransport;
use SendableEmail;
use native_tls::TlsConnector;
use smtp::authentication::{Credentials, Mechanism, DEFAULT_ENCRYPTED_MECHANISMS,
                           DEFAULT_UNENCRYPTED_MECHANISMS};
use smtp::client::Client;
use smtp::client::net::ClientTlsParameters;
use smtp::client::net::DEFAULT_TLS_PROTOCOLS;
use smtp::commands::*;
use smtp::error::{Error, SmtpResult};
use smtp::extension::{ClientId, Extension, MailBodyParameter, MailParameter, ServerInfo};
use std::io::Read;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

pub mod extension;
pub mod commands;
pub mod authentication;
pub mod response;
pub mod client;
pub mod error;
pub mod util;

// Registrated port numbers:
// https://www.iana.
// org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml

/// Default smtp port
pub const SMTP_PORT: u16 = 25;

/// Default submission port
pub const SUBMISSION_PORT: u16 = 587;

// Useful strings and characters

/// The word separator for SMTP transactions
pub const SP: &str = " ";

/// The line ending for SMTP transactions (carriage return + line feed)
pub const CRLF: &str = "\r\n";

/// Colon
pub const COLON: &str = ":";

/// The ending of message content
pub const MESSAGE_ENDING: &str = "\r\n.\r\n";

/// NUL unicode character
pub const NUL: &str = "\0";

/// How to apply TLS to a client connection
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub enum ClientSecurity {
    /// Insecure connection
    None,
    /// Use `STARTTLS` when available
    Opportunistic(ClientTlsParameters),
    /// Always use `STARTTLS`
    Required(ClientTlsParameters),
    /// Use TLS wrapped connection without negotation
    /// Non RFC-compliant, should only be used if the server does not support STARTTLS.
    Wrapper(ClientTlsParameters),
}

/// Configures connection reuse behavior
#[derive(Clone, Debug, Copy)]
#[cfg_attr(feature = "serde-impls", derive(Serialize, Deserialize))]
pub enum ConnectionReuseParameters {
    /// Unlimitied connection reuse
    ReuseUnlimited,
    /// Maximum number of connection reuse
    ReuseLimited(u16),
    /// Disable connection reuse, close connection after each transaction
    NoReuse,
}

/// Contains client configuration
#[allow(missing_debug_implementations)]
pub struct SmtpTransportBuilder {
    /// Enable connection reuse
    connection_reuse: ConnectionReuseParameters,
    /// Name sent during EHLO
    hello_name: ClientId,
    /// Credentials
    credentials: Option<Credentials>,
    /// Socket we are connecting to
    server_addr: SocketAddr,
    /// TLS security configuration
    security: ClientSecurity,
    /// Enable UTF8 mailboxes in envelope or headers
    smtp_utf8: bool,
    /// Optional enforced authentication mechanism
    authentication_mechanism: Option<Mechanism>,
    /// Define network timeout
    /// It can be changed later for specific needs (like a different timeout for each SMTP command)
    timeout: Option<Duration>,
}

/// Builder for the SMTP `SmtpTransport`
impl SmtpTransportBuilder {
    /// Creates a new SMTP client
    ///
    /// Defaults are:
    ///
    /// * No connection reuse
    /// * No authentication
    /// * No SMTPUTF8 support
    /// * A 60 seconds timeout for smtp commands
    pub fn new<A: ToSocketAddrs>(
        addr: A,
        security: ClientSecurity,
    ) -> Result<SmtpTransportBuilder, Error> {
        let mut addresses = addr.to_socket_addrs()?;

        match addresses.next() {
            Some(addr) => Ok(SmtpTransportBuilder {
                server_addr: addr,
                security,
                smtp_utf8: false,
                credentials: None,
                connection_reuse: ConnectionReuseParameters::NoReuse,
                hello_name: ClientId::hostname(),
                authentication_mechanism: None,
                timeout: Some(Duration::new(60, 0)),
            }),
            None => Err(Error::Resolution),
        }
    }

    /// Enable SMTPUTF8 if the server supports it
    pub fn smtp_utf8(mut self, enabled: bool) -> SmtpTransportBuilder {
        self.smtp_utf8 = enabled;
        self
    }

    /// Set the name used during EHLO
    pub fn hello_name(mut self, name: ClientId) -> SmtpTransportBuilder {
        self.hello_name = name;
        self
    }

    /// Enable connection reuse
    pub fn connection_reuse(
        mut self,
        parameters: ConnectionReuseParameters,
    ) -> SmtpTransportBuilder {
        self.connection_reuse = parameters;
        self
    }

    /// Set the client credentials
    pub fn credentials<S: Into<Credentials>>(mut self, credentials: S) -> SmtpTransportBuilder {
        self.credentials = Some(credentials.into());
        self
    }

    /// Set the authentication mechanism to use
    pub fn authentication_mechanism(mut self, mechanism: Mechanism) -> SmtpTransportBuilder {
        self.authentication_mechanism = Some(mechanism);
        self
    }

    /// Set the timeout duration
    pub fn timeout(mut self, timeout: Option<Duration>) -> SmtpTransportBuilder {
        self.timeout = timeout;
        self
    }

    /// Build the SMTP client
    ///
    /// It does not connect to the server, but only creates the `SmtpTransport`
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
#[allow(missing_debug_implementations)]
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
                    $client.close();
                }
                return Err(From::from(err))
            },
        }
    })
);

impl<'a> SmtpTransport {
    /// Simple and secure transport, should be used when possible.
    /// Creates an encrypted transport over submission port, using the provided domain
    /// to validate TLS certificates.
    pub fn simple_builder(domain: &str) -> Result<SmtpTransportBuilder, Error> {
        let mut tls_builder = TlsConnector::builder()?;
        tls_builder.supported_protocols(DEFAULT_TLS_PROTOCOLS)?;

        let tls_parameters =
            ClientTlsParameters::new(domain.to_string(), tls_builder.build().unwrap());

        SmtpTransportBuilder::new(
            (domain, SUBMISSION_PORT),
            ClientSecurity::Required(tls_parameters),
        )
    }

    /// Creates a new configurable builder
    pub fn builder<A: ToSocketAddrs>(
        addr: A,
        security: ClientSecurity,
    ) -> Result<SmtpTransportBuilder, Error> {
        SmtpTransportBuilder::new(addr, security)
    }

    /// Creates a new local SMTP client to port 25
    pub fn builder_unencrypted_localhost() -> Result<SmtpTransportBuilder, Error> {
        SmtpTransportBuilder::new(("localhost", SMTP_PORT), ClientSecurity::None)
    }

    /// Creates a new SMTP client
    ///
    /// It does not connect to the server, but only creates the `SmtpTransport`
    pub fn new(builder: SmtpTransportBuilder) -> SmtpTransport {
        let client = Client::new();

        SmtpTransport {
            client,
            server_info: None,
            client_info: builder,
            state: State {
                panic: false,
                connection_reuse_count: 0,
            },
        }
    }

    /// Gets the EHLO response and updates server information
    fn ehlo(&mut self) -> SmtpResult {
        // Extended Hello
        let ehlo_response = try_smtp!(
            self.client.command(EhloCommand::new(ClientId::new(
                self.client_info.hello_name.to_string()
            ),)),
            self
        );

        self.server_info = Some(try_smtp!(ServerInfo::from_response(&ehlo_response), self));

        // Print server information
        debug!("server {}", self.server_info.as_ref().unwrap());

        Ok(ehlo_response)
    }

    /// Reset the client state
    pub fn close(&mut self) {
        // Close the SMTP transaction if needed
        self.client.close();

        // Reset the client state
        self.server_info = None;
        self.state.panic = false;
        self.state.connection_reuse_count = 0;
    }
}

impl<'a, T: Read + 'a> EmailTransport<'a, T, SmtpResult> for SmtpTransport {
    /// Sends an email
    #[cfg_attr(feature = "cargo-clippy", allow(match_same_arms, cyclomatic_complexity))]
    fn send<U: SendableEmail<'a, T> + 'a>(&mut self, email: &'a U) -> SmtpResult {
        // Extract email information
        let message_id = email.message_id();
        let envelope = email.envelope();

        // Check if the connection is still available
        if (self.state.connection_reuse_count > 0) && (!self.client.is_connected()) {
            self.close();
        }

        if self.state.connection_reuse_count == 0 {
            self.client.connect(
                &self.client_info.server_addr,
                match self.client_info.security {
                    ClientSecurity::Wrapper(ref tls_parameters) => Some(tls_parameters),
                    _ => None,
                },
            )?;

            self.client.set_timeout(self.client_info.timeout)?;

            // Log the connection
            info!("connection established to {}", self.client_info.server_addr);

            self.ehlo()?;

            match (
                &self.client_info.security.clone(),
                self.server_info
                    .as_ref()
                    .unwrap()
                    .supports_feature(Extension::StartTls),
            ) {
                (&ClientSecurity::Required(_), false) => {
                    return Err(From::from("Could not encrypt connection, aborting"))
                }
                (&ClientSecurity::Opportunistic(_), false) => (),
                (&ClientSecurity::None, _) => (),
                (&ClientSecurity::Wrapper(_), _) => (),
                (&ClientSecurity::Opportunistic(ref tls_parameters), true)
                | (&ClientSecurity::Required(ref tls_parameters), true) => {
                    try_smtp!(self.client.command(StarttlsCommand), self);
                    try_smtp!(self.client.upgrade_tls_stream(tls_parameters), self);

                    debug!("connection encrypted");

                    // Send EHLO again
                    self.ehlo()?;
                }
            }

            if self.client_info.credentials.is_some() {
                let mut found = false;

                // Compute accepted mechanism
                let accepted_mechanisms = match self.client_info.authentication_mechanism {
                    Some(mechanism) => vec![mechanism],
                    None => {
                        if self.client.is_encrypted() {
                            DEFAULT_ENCRYPTED_MECHANISMS.to_vec()
                        } else {
                            DEFAULT_UNENCRYPTED_MECHANISMS.to_vec()
                        }
                    }
                };

                for mechanism in accepted_mechanisms {
                    if self.server_info
                        .as_ref()
                        .unwrap()
                        .supports_auth_mechanism(mechanism)
                    {
                        found = true;
                        try_smtp!(
                            self.client
                                .auth(mechanism, self.client_info.credentials.as_ref().unwrap(),),
                            self
                        );
                        break;
                    }
                }

                if !found {
                    info!("No supported authentication mechanisms available");
                }
            }
        }

        // Mail
        let mut mail_options = vec![];

        if self.server_info
            .as_ref()
            .unwrap()
            .supports_feature(Extension::EightBitMime)
        {
            mail_options.push(MailParameter::Body(MailBodyParameter::EightBitMime));
        }

        if self.server_info
            .as_ref()
            .unwrap()
            .supports_feature(Extension::SmtpUtfEight) && self.client_info.smtp_utf8
        {
            mail_options.push(MailParameter::SmtpUtfEight);
        }

        try_smtp!(
            self.client
                .command(MailCommand::new(envelope.from().cloned(), mail_options,)),
            self
        );

        // Log the mail command
        info!(
            "{}: from=<{}>",
            message_id,
            match envelope.from() {
                Some(address) => address.to_string(),
                None => "".to_string(),
            }
        );

        // Recipient
        for to_address in envelope.to() {
            try_smtp!(
                self.client
                    .command(RcptCommand::new(to_address.clone(), vec![]),),
                self
            );
            // Log the rcpt command
            info!("{}: to=<{}>", message_id, to_address);
        }

        // Data
        try_smtp!(self.client.command(DataCommand), self);

        // Message content
        let result = self.client.message(email.message());

        if result.is_ok() {
            // Increment the connection reuse counter
            self.state.connection_reuse_count += 1;

            // Log the message
            info!(
                "{}: conn_use={}, status=sent ({})",
                message_id,
                self.state.connection_reuse_count,
                result
                    .as_ref()
                    .ok()
                    .unwrap()
                    .message
                    .iter()
                    .next()
                    .unwrap_or(&"no response".to_string())
            );
        }

        // Test if we can reuse the existing connection
        match self.client_info.connection_reuse {
            ConnectionReuseParameters::ReuseLimited(limit)
                if self.state.connection_reuse_count >= limit =>
            {
                self.close()
            }
            ConnectionReuseParameters::NoReuse => self.close(),
            _ => (),
        }

        result
    }
}
