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
//! * AUTH ([RFC 4954](http://tools.ietf.org/html/rfc4954)) with PLAIN, LOGIN and XOAUTH2 mechanisms
//! * STARTTLS ([RFC 2487](http://tools.ietf.org/html/rfc2487))
//! * SMTPUTF8 ([RFC 6531](http://tools.ietf.org/html/rfc6531))
//!

use crate::Envelope;
use crate::{
    transport::smtp::{
        authentication::{
            Credentials, Mechanism, DEFAULT_ENCRYPTED_MECHANISMS, DEFAULT_UNENCRYPTED_MECHANISMS,
        },
        client::{net::ClientTlsParameters, SmtpConnection},
        commands::*,
        error::{Error, SmtpResult},
        extension::{ClientId, Extension, MailBodyParameter, MailParameter, ServerInfo},
    },
    Transport,
};
use log::{debug, info};
#[cfg(feature = "native-tls")]
use native_tls::{Protocol, TlsConnector};
#[cfg(feature = "rustls")]
use rustls::ClientConfig;
use std::{
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};
use uuid::Uuid;
#[cfg(feature = "rustls")]
use webpki_roots::TLS_SERVER_ROOTS;

pub mod authentication;
pub mod client;
pub mod commands;
pub mod error;
pub mod extension;
#[cfg(feature = "connection-pool")]
pub mod r2d2;
pub mod response;
pub mod util;

// Registered port numbers:
// https://www.iana.
// org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml

/// Default smtp port
pub const SMTP_PORT: u16 = 25;
/// Default submission port
pub const SUBMISSION_PORT: u16 = 587;
/// Default submission over TLS port
pub const SUBMISSIONS_PORT: u16 = 465;

/// Accepted protocols by default.
/// This removes TLS 1.0 and 1.1 compared to tls-native defaults.
/// This is also rustls' default behavior
#[cfg(feature = "native-tls")]
const DEFAULT_TLS_MIN_PROTOCOL: Protocol = Protocol::Tlsv12;

/// How to apply TLS to a client connection
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub enum ClientSecurity {
    /// Insecure connection only (for testing purposes)
    None,
    /// Start with insecure connection and use `STARTTLS` when available
    Opportunistic(ClientTlsParameters),
    /// Start with insecure connection and require `STARTTLS`
    Required(ClientTlsParameters),
    /// Use TLS wrapped connection
    Wrapper(ClientTlsParameters),
}

/// Configures connection reuse behavior
#[derive(Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConnectionReuseParameters {
    /// Unlimited connection reuse
    ReuseUnlimited,
    /// Maximum number of connection reuse
    ReuseLimited(u16),
    /// Disable connection reuse, close connection after each transaction
    NoReuse,
}

/// Contains client configuration
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct SmtpClient {
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
    /// Force use of the set authentication mechanism even if server does not report to support it
    force_set_auth: bool,
    /// Define network timeout
    /// It can be changed later for specific needs (like a different timeout for each SMTP command)
    timeout: Option<Duration>,
}

/// Builder for the SMTP `SmtpTransport`
impl SmtpClient {
    /// Creates a new SMTP client
    ///
    /// Defaults are:
    ///
    /// * No connection reuse
    /// * No authentication
    /// * No SMTPUTF8 support
    /// * A 60 seconds timeout for smtp commands
    ///
    /// Consider using [`SmtpClient::new_simple`] instead, if possible.
    pub fn new<A: ToSocketAddrs>(addr: A, security: ClientSecurity) -> Result<SmtpClient, Error> {
        let mut addresses = addr.to_socket_addrs()?;

        match addresses.next() {
            Some(addr) => Ok(SmtpClient {
                server_addr: addr,
                security,
                smtp_utf8: false,
                credentials: None,
                connection_reuse: ConnectionReuseParameters::NoReuse,
                #[cfg(feature = "hostname")]
                hello_name: ClientId::hostname(),
                #[cfg(not(feature = "hostname"))]
                hello_name: ClientId::new("localhost".to_string()),
                authentication_mechanism: None,
                force_set_auth: false,
                timeout: Some(Duration::new(60, 0)),
            }),
            None => Err(Error::Resolution),
        }
    }

    /// Simple and secure transport, should be used when possible.
    /// Creates an encrypted transport over submissions port, using the provided domain
    /// to validate TLS certificates.
    #[cfg(feature = "native-tls")]
    pub fn new_simple(domain: &str) -> Result<SmtpClient, Error> {
        let mut tls_builder = TlsConnector::builder();
        tls_builder.min_protocol_version(Some(DEFAULT_TLS_MIN_PROTOCOL));

        let tls_parameters =
            ClientTlsParameters::new(domain.to_string(), tls_builder.build().unwrap());

        SmtpClient::new(
            (domain, SUBMISSIONS_PORT),
            ClientSecurity::Wrapper(tls_parameters),
        )
    }

    #[cfg(feature = "rustls")]
    pub fn new_simple(domain: &str) -> Result<SmtpClient, Error> {
        let mut tls = ClientConfig::new();
        tls.config
            .root_store
            .add_server_trust_anchors(&TLS_SERVER_ROOTS);

        let tls_parameters = ClientTlsParameters::new(domain.to_string(), tls);

        SmtpClient::new(
            (domain, SUBMISSIONS_PORT),
            ClientSecurity::Wrapper(tls_parameters),
        )
    }

    /// Creates a new local SMTP client to port 25
    pub fn new_unencrypted_localhost() -> Result<SmtpClient, Error> {
        SmtpClient::new(("localhost", SMTP_PORT), ClientSecurity::None)
    }

    /// Enable SMTPUTF8 if the server supports it
    pub fn smtp_utf8(mut self, enabled: bool) -> SmtpClient {
        self.smtp_utf8 = enabled;
        self
    }

    /// Set the name used during EHLO
    pub fn hello_name(mut self, name: ClientId) -> SmtpClient {
        self.hello_name = name;
        self
    }

    /// Enable connection reuse
    pub fn connection_reuse(mut self, parameters: ConnectionReuseParameters) -> SmtpClient {
        self.connection_reuse = parameters;
        self
    }

    /// Set the client credentials
    pub fn credentials<S: Into<Credentials>>(mut self, credentials: S) -> SmtpClient {
        self.credentials = Some(credentials.into());
        self
    }

    /// Set the authentication mechanism to use
    pub fn authentication_mechanism(mut self, mechanism: Mechanism) -> SmtpClient {
        self.authentication_mechanism = Some(mechanism);
        self
    }

    /// Set if the set authentication mechanism should be force
    pub fn force_set_auth(mut self, force: bool) -> SmtpClient {
        self.force_set_auth = force;
        self
    }

    /// Set the timeout duration
    pub fn timeout(mut self, timeout: Option<Duration>) -> SmtpClient {
        self.timeout = timeout;
        self
    }

    /// Build the SMTP client
    ///
    /// It does not connect to the server, but only creates the `SmtpTransport`
    pub fn transport(self) -> SmtpTransport {
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
    /// Value is None before EHLO
    server_info: Option<ServerInfo>,
    /// SmtpTransport variable states
    state: State,
    /// Information about the client
    client_info: SmtpClient,
    /// Low level client
    client: SmtpConnection,
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
    /// Creates a new SMTP client
    ///
    /// It does not connect to the server, but only creates the `SmtpTransport`
    pub fn new(builder: SmtpClient) -> SmtpTransport {
        let client = SmtpConnection::new();

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

    fn connect(&mut self) -> Result<(), Error> {
        // Check if the connection is still available
        if (self.state.connection_reuse_count > 0) && (!self.client.is_connected()) {
            self.close();
        }

        if self.state.connection_reuse_count > 0 {
            info!(
                "connection already established to {}",
                self.client_info.server_addr
            );
            return Ok(());
        }

        self.client.connect(
            &self.client_info.server_addr,
            self.client_info.timeout,
            match self.client_info.security {
                ClientSecurity::Wrapper(ref tls_parameters) => Some(tls_parameters),
                _ => None,
            },
        )?;

        self.client.set_timeout(self.client_info.timeout)?;
        let _response = self.client.read_response()?;

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
                return Err(From::from("Could not encrypt connection, aborting"));
            }
            (&ClientSecurity::Opportunistic(_), false) => (),
            (&ClientSecurity::None, _) => (),
            (&ClientSecurity::Wrapper(_), _) => (),
            #[cfg(any(feature = "native-tls", feature = "rustls"))]
            (&ClientSecurity::Opportunistic(ref tls_parameters), true)
            | (&ClientSecurity::Required(ref tls_parameters), true) => {
                try_smtp!(self.client.command(StarttlsCommand), self);
                try_smtp!(self.client.upgrade_tls_stream(tls_parameters), self);
                debug!("connection encrypted");
                // Send EHLO again
                self.ehlo()?;
            }
            #[cfg(not(any(feature = "native-tls", feature = "rustls")))]
            (&ClientSecurity::Opportunistic(_), true) | (&ClientSecurity::Required(_), true) => {
                // This should never happen as `ClientSecurity` can only be created
                // when a TLS library is enabled
                unreachable!("TLS support required but not supported");
            }
        }

        if self.client_info.credentials.is_some() {
            let mut found = false;

            if !self.client_info.force_set_auth {
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
                    if self
                        .server_info
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
            } else {
                try_smtp!(
                    self.client.auth(
                        self.client_info.authentication_mechanism.expect(
                            "force_set_auth set to true, but no authentication mechanism set"
                        ),
                        self.client_info.credentials.as_ref().unwrap(),
                    ),
                    self
                );
                found = true;
            }

            if !found {
                info!("No supported authentication mechanisms available");
            }
        }
        Ok(())
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

impl<'a> Transport<'a> for SmtpTransport {
    type Result = SmtpResult;

    /// Sends an email
    #[cfg_attr(
        feature = "cargo-clippy",
        allow(clippy::match_same_arms, clippy::cyclomatic_complexity)
    )]
    fn send_raw(&mut self, envelope: &Envelope, email: &[u8]) -> Self::Result {
        let email_id = Uuid::new_v4();
        let envelope = envelope;

        if !self.client.is_connected() {
            self.connect()?;
        }

        // Mail
        let mut mail_options = vec![];

        if self
            .server_info
            .as_ref()
            .unwrap()
            .supports_feature(Extension::EightBitMime)
        {
            mail_options.push(MailParameter::Body(MailBodyParameter::EightBitMime));
        }

        if self
            .server_info
            .as_ref()
            .unwrap()
            .supports_feature(Extension::SmtpUtfEight)
            && self.client_info.smtp_utf8
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
            email_id,
            match envelope.from() {
                Some(address) => address.to_string(),
                None => "".to_string(),
            }
        );

        // Recipient
        for to_address in envelope.to() {
            try_smtp!(
                self.client
                    .command(RcptCommand::new(to_address.clone(), vec![])),
                self
            );
            // Log the rcpt command
            info!("{}: to=<{}>", email_id, to_address);
        }

        // Data
        try_smtp!(self.client.command(DataCommand), self);

        // Message content
        let result = self.client.message(email);

        if let Ok(ref result) = result {
            // Increment the connection reuse counter
            self.state.connection_reuse_count += 1;

            // Log the message
            info!(
                "{}: conn_use={}, status=sent ({})",
                email_id,
                self.state.connection_reuse_count,
                result
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
