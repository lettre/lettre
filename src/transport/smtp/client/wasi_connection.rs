#![cfg(all(target_arch = "wasm32", feature = "wasi"))]
use wasip3::wit_bindgen::StreamResult;

use crate::{
    address::Envelope,
    transport::smtp::{
        client::{wasi_net::WasiNetworkStream, ClientCodec},
        commands::{Data, Ehlo, Mail, Quit, Rcpt},
        error::{self, Error},
        extension::{ClientId, Extension, MailBodyParameter, MailParameter, ServerInfo},
        response::{parse_response, Response},
    },
};
use std::{fmt::Display, time::Duration};

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
// TODO : use SocketAddr as optional connection method in cases where we don't need hostname resolution
// pub enum SocketAddr {
//     /// An IPv4 socket address.
//     V4(wasip3::sockets::types::Ipv4SocketAddress),
//     /// An IPv6 socket address.
//     V6(wasip3::sockets::types::Ipv6SocketAddress),
// }

pub struct WasiSmtpConnection {
    stream: WasiNetworkStream,
    panic: bool,
    server_info: ServerInfo,
}

impl WasiSmtpConnection {
    pub fn server_info(&self) -> &ServerInfo {
        &self.server_info
    }
    pub async fn connect_wasi(
        host: &str,
        port: u16,
        timeout: Option<Duration>,
        hello_name: &ClientId,
    ) -> Result<WasiSmtpConnection, Error> {
        // case : do we need support for non-hostname workflow ? ip-address passing ?
        let stream = WasiNetworkStream::connect_wasi(host, port, timeout).await?;
        Self::connect_impl(stream, hello_name).await
    }

    async fn connect_impl(
        stream: WasiNetworkStream,
        hello_name: &ClientId,
    ) -> Result<WasiSmtpConnection, Error> {
        let mut conn = WasiSmtpConnection {
            stream,
            panic: false,
            server_info: ServerInfo::default(),
        };

        let _response = conn.read_response().await?;

        conn.ehlo(hello_name).await?;

        // Print server information
        #[cfg(feature = "tracing")]
        tracing::debug!("server {}", conn.server_info);
        Ok(conn)
    }

    pub async fn send(&mut self, envelope: &Envelope, email: &[u8]) -> Result<Response, Error> {
        let mut mail_options: Vec<MailParameter> = vec![];
        eprintln!("wasip3: Sending mail");
        if envelope.has_non_ascii_addresses() {
            if !self.server_info().supports_feature(Extension::SmtpUtfEight) {
                return Err(error::client(
                    "Envelope contains non-ascii chars but server does not support SMTPUTF8",
                ));
            }
            mail_options.push(MailParameter::SmtpUtfEight);
        }

        if !email.is_ascii() {
            if !self.server_info().supports_feature(Extension::EightBitMime) {
                return Err(error::client(
                    "Message contains non-ascii chars but server does not support 8BITMIME",
                ));
            }
            mail_options.push(MailParameter::Body(MailBodyParameter::EightBitMime));
        }

        try_smtp!(
            self.command(Mail::new(envelope.from().cloned(), mail_options))
                .await,
            self
        );
        eprintln!("wasip3: logger 2");

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
        eprintln!("wasip3: Message content");
        Ok(result)
    }

    /// sends an SMTP command
    pub async fn command<C: Display>(&mut self, command: C) -> Result<Response, Error> {
        // write to socket
        self.write(command.to_string().as_bytes()).await?;
        // read response
        self.read_response().await
    }

    /// Writing string to the server
    /// Wasi `write_all` returns (Optional) remaining bytes that weren't written
    async fn write(&mut self, string: &[u8]) -> Result<(), Error> {
        let remaining = self.stream.writer.write_all(string.to_vec()).await;
        eprintln!("wasip3: writing bytes : {:?}", string.to_vec());

        if !remaining.is_empty() {
            return Err(error::network(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!(
                    "Failed to write all data: {} bytes remaining",
                    remaining.len()
                ),
            )));
        }

        #[cfg(feature = "tracing")]
        tracing::debug!(">> {}", escape_crlf(&String::from_utf8_lossy(string)));

        Ok(())
    }
    // there's no read_line function for the client_rx, instead we need to buffer and manually parse
    // response in chunks and seggregate them based on \r\n delimiter to structure a line
    // we then parse the response as usual.
    pub async fn read_response(&mut self) -> Result<Response, Error> {
        let mut full_response = String::with_capacity(100);
        let mut line_buffer = Vec::new();

        loop {
            // Read a chunk
            let (result, mut chunk_data) = self.stream.reader.read(Vec::with_capacity(256)).await;

            match result {
                StreamResult::Complete(n) if n > 0 => {
                    line_buffer.append(&mut chunk_data);

                    // Process complete lines (SMTP uses \r\n)
                    loop {
                        if let Some(pos) = self.find_crlf(&line_buffer) {
                            // Extract line including \r\n
                            let line_bytes: Vec<u8> = line_buffer.drain(..=pos + 1).collect();
                            let line = String::from_utf8_lossy(&line_bytes);

                            #[cfg(feature = "tracing")]
                            tracing::debug!("<< {}", escape_crlf(&line));

                            full_response.push_str(&line);

                            // Try to parse the accumulated response
                            match parse_response(&full_response) {
                                Ok((_remaining, response)) => {
                                    return if response.is_positive() {
                                        Ok(response)
                                    } else {
                                        Err(error::code(
                                            response.code(),
                                            Some(response.message().collect()),
                                        ))
                                    };
                                }
                                Err(nom::Err::Incomplete(_)) => {
                                    // Multi-line response, keep reading
                                    continue;
                                }
                                Err(nom::Err::Failure(e)) | Err(nom::Err::Error(e)) => {
                                    return Err(error::response(e.to_string()));
                                }
                            }
                        } else {
                            // No complete line yet, break inner loop to read more data
                            break;
                        }
                    }
                }
                StreamResult::Complete(_) => {
                    // Complete(0) - Stream ended (EOF)
                    if !full_response.is_empty() {
                        // Try to parse what we have
                        match parse_response(&full_response) {
                            Ok((_remaining, response)) => {
                                return if response.is_positive() {
                                    Ok(response)
                                } else {
                                    Err(error::code(
                                        response.code(),
                                        Some(response.message().collect()),
                                    ))
                                };
                            }
                            _ => {}
                        }
                    }
                    return Err(error::response("incomplete response: stream ended"));
                }
                StreamResult::Dropped => {
                    // Stream was dropped by peer
                    return Err(error::response("stream dropped by peer"));
                }
                StreamResult::Cancelled => {
                    // Stream was cancelled
                    return Err(error::response("stream cancelled"));
                }
            }
        }
    }

    // Helper function to find \r\n in buffer
    pub fn find_crlf(&mut self, buffer: &[u8]) -> Option<usize> {
        buffer.windows(2).position(|window| window == b"\r\n")
    }

    async fn ehlo(&mut self, hello_name: &ClientId) -> Result<(), Error> {
        let ehlo_response = try_smtp!(self.command(Ehlo::new(hello_name.clone())).await, self);
        eprintln!("wasip3: ehlo Successful");
        self.server_info = try_smtp!(ServerInfo::from_response(&ehlo_response), self);
        Ok(())
    }

    pub async fn abort(&mut self) {
        // Only try to quit if we are not already broken
        if !self.panic {
            self.panic = true;
            let _ = self.command(Quit).await;
        }
        // drop(&mut self.stream.reader);
        // drop(&mut self.stream.writer);
        if let Some(finish_future) = self.stream.finish_future.take() {
            finish_future.await.ok();
        }
    }

    pub async fn message(&mut self, message: &[u8]) -> Result<Response, Error> {
        let mut out_buf: Vec<u8> = vec![];
        let mut codec = ClientCodec::new();
        codec.encode(message, &mut out_buf);
        self.write(out_buf.as_slice()).await?;
        self.write(b"\r\n.\r\n").await?;
        self.read_response().await
    }
}
