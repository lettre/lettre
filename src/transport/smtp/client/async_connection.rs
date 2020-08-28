use std::{fmt::Display, io};

use futures_util::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::{AsyncNetworkStream, ClientCodec, TlsParameters};
use crate::{
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        commands::*,
        error::Error,
        extension::{ClientId, Extension, MailBodyParameter, MailParameter, ServerInfo},
        response::{parse_response, Response},
    },
    Envelope,
};

#[cfg(feature = "tracing")]
use super::escape_crlf;

macro_rules! try_smtp (
    ($err: expr, $client: ident) => ({
        match $err {
            Ok(val) => val,
            Err(err) => {
                $client.abort().await;
                return Err(From::from(err))
            },
        }
    })
);

/// Structure that implements the SMTP client
pub struct AsyncSmtpConnection {
    /// TCP stream between client and server
    /// Value is None before connection
    stream: BufReader<AsyncNetworkStream>,
    /// Panic state
    panic: bool,
    /// Information about the server
    server_info: ServerInfo,
}

impl AsyncSmtpConnection {
    pub fn server_info(&self) -> &ServerInfo {
        &self.server_info
    }

    // FIXME add simple connect and rename this one

    /// Connects to the configured server
    ///
    /// Sends EHLO and parses server information
    pub async fn connect_tokio02(
        hostname: &str,
        port: u16,
        hello_name: &ClientId,
        tls_parameters: Option<TlsParameters>,
    ) -> Result<AsyncSmtpConnection, Error> {
        let stream = AsyncNetworkStream::connect_tokio02(hostname, port, tls_parameters).await?;
        Self::connect_impl(stream, hello_name).await
    }

    async fn connect_impl(
        stream: AsyncNetworkStream,
        hello_name: &ClientId,
    ) -> Result<AsyncSmtpConnection, Error> {
        let stream = BufReader::new(stream);
        let mut conn = AsyncSmtpConnection {
            stream,
            panic: false,
            server_info: ServerInfo::default(),
        };
        // TODO log
        let _response = conn.read_response().await?;

        conn.ehlo(hello_name).await?;

        // Print server information
        #[cfg(feature = "tracing")]
        tracing::debug!("server {}", conn.server_info);
        Ok(conn)
    }

    pub async fn send(&mut self, envelope: &Envelope, email: &[u8]) -> Result<Response, Error> {
        // Mail
        let mut mail_options = vec![];

        if self.server_info().supports_feature(Extension::EightBitMime) {
            mail_options.push(MailParameter::Body(MailBodyParameter::EightBitMime));
        }
        try_smtp!(
            self.command(Mail::new(envelope.from().cloned(), mail_options))
                .await,
            self
        );

        // Recipient
        for to_address in envelope.to() {
            try_smtp!(
                self.command(Rcpt::new(to_address.clone(), vec![])).await,
                self
            );
        }

        // Data
        try_smtp!(self.command(Data).await, self);

        // Message content
        let result = try_smtp!(self.message(email).await, self);
        Ok(result)
    }

    pub fn has_broken(&self) -> bool {
        self.panic
    }

    pub fn can_starttls(&self) -> bool {
        !self.is_encrypted() && self.server_info.supports_feature(Extension::StartTls)
    }

    #[allow(unused_variables)]
    pub async fn starttls(
        &mut self,
        tls_parameters: TlsParameters,
        hello_name: &ClientId,
    ) -> Result<(), Error> {
        if self.server_info.supports_feature(Extension::StartTls) {
            try_smtp!(self.command(Starttls).await, self);
            try_smtp!(
                self.stream.get_mut().upgrade_tls(tls_parameters).await,
                self
            );
            #[cfg(feature = "tracing")]
            tracing::debug!("connection encrypted");
            // Send EHLO again
            try_smtp!(self.ehlo(hello_name).await, self);
            Ok(())
        } else {
            Err(Error::Client("STARTTLS is not supported on this server"))
        }
    }

    /// Send EHLO and update server info
    async fn ehlo(&mut self, hello_name: &ClientId) -> Result<(), Error> {
        let ehlo_response = try_smtp!(self.command(Ehlo::new(hello_name.clone())).await, self);
        self.server_info = try_smtp!(ServerInfo::from_response(&ehlo_response), self);
        Ok(())
    }

    pub async fn quit(&mut self) -> Result<Response, Error> {
        Ok(try_smtp!(self.command(Quit).await, self))
    }

    pub async fn abort(&mut self) {
        // Only try to quit if we are not already broken
        if !self.panic {
            self.panic = true;
            let _ = self.command(Quit).await;
        }
    }

    /// Sets the underlying stream
    pub fn set_stream(&mut self, stream: AsyncNetworkStream) {
        self.stream = BufReader::new(stream);
    }

    /// Tells if the underlying stream is currently encrypted
    pub fn is_encrypted(&self) -> bool {
        self.stream.get_ref().is_encrypted()
    }

    /// Checks if the server is connected using the NOOP SMTP command
    pub async fn test_connected(&mut self) -> bool {
        self.command(Noop).await.is_ok()
    }

    /// Sends an AUTH command with the given mechanism, and handles challenge if needed
    pub async fn auth(
        &mut self,
        mechanisms: &[Mechanism],
        credentials: &Credentials,
    ) -> Result<Response, Error> {
        let mechanism = self
            .server_info
            .get_auth_mechanism(mechanisms)
            .ok_or(Error::Client(
                "No compatible authentication mechanism was found",
            ))?;

        // Limit challenges to avoid blocking
        let mut challenges = 10;
        let mut response = self
            .command(Auth::new(mechanism, credentials.clone(), None)?)
            .await?;

        while challenges > 0 && response.has_code(334) {
            challenges -= 1;
            response = try_smtp!(
                self.command(Auth::new_from_response(
                    mechanism,
                    credentials.clone(),
                    &response,
                )?)
                .await,
                self
            );
        }

        if challenges == 0 {
            Err(Error::ResponseParsing("Unexpected number of challenges"))
        } else {
            Ok(response)
        }
    }

    /// Sends the message content
    pub async fn message(&mut self, message: &[u8]) -> Result<Response, Error> {
        let mut out_buf: Vec<u8> = vec![];
        let mut codec = ClientCodec::new();
        codec.encode(message, &mut out_buf);
        self.write(out_buf.as_slice()).await?;
        self.write(b"\r\n.\r\n").await?;
        self.read_response().await
    }

    /// Sends an SMTP command
    pub async fn command<C: Display>(&mut self, command: C) -> Result<Response, Error> {
        self.write(command.to_string().as_bytes()).await?;
        self.read_response().await
    }

    /// Writes a string to the server
    async fn write(&mut self, string: &[u8]) -> Result<(), Error> {
        self.stream.get_mut().write_all(string).await?;
        self.stream.get_mut().flush().await?;

        #[cfg(feature = "tracing")]
        tracing::debug!("Wrote: {}", escape_crlf(&String::from_utf8_lossy(string)));
        Ok(())
    }

    /// Gets the SMTP response
    pub async fn read_response(&mut self) -> Result<Response, Error> {
        let mut buffer = String::with_capacity(100);

        while self.stream.read_line(&mut buffer).await? > 0 {
            #[cfg(feature = "tracing")]
            tracing::debug!("<< {}", escape_crlf(&buffer));
            match parse_response(&buffer) {
                Ok((_remaining, response)) => {
                    if response.is_positive() {
                        return Ok(response);
                    }

                    return Err(response.into());
                }
                Err(nom::Err::Failure(e)) => {
                    return Err(Error::Parsing(e.1));
                }
                Err(nom::Err::Incomplete(_)) => { /* read more */ }
                Err(nom::Err::Error(e)) => {
                    return Err(Error::Parsing(e.1));
                }
            }
        }

        Err(io::Error::new(io::ErrorKind::Other, "incomplete").into())
    }
}
