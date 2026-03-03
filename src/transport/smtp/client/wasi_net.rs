//! WASI-compatible async network stream for lettre (wasip3)
//! Using wit_bindgen::rt::async_support::spawn for background send task

//#![cfg(all(target_arch = "wasm32", feature = "wasi"))]
use std::time::Duration;

use crate::transport::smtp::client::TlsParameters;
use crate::transport::smtp::{error, Error};
use wasip3::sockets::types::IpSocketAddress;
use wasip3::wit_bindgen::rt::async_support::{spawn, FutureReader, StreamReader, StreamWriter};
use wasip3::{sockets::types::ErrorCode, wit_stream};

/// WASI-compatible network stream for lettre — concrete halves from the host
pub struct WasiNetworkStream {
    /// Component -> Host writer (guest writes bytes here)
    pub writer: StreamWriter<u8>,
    /// Host -> Component reader (guest reads bytes from here)
    pub reader: StreamReader<u8>,
    /// Optional future returned by `TcpSocket::receive()` that the guest should
    /// await or drive to completion when closing the connection
    pub finish_future: Option<FutureReader<Result<(), ErrorCode>>>,
}

impl WasiNetworkStream {
    pub fn from_halves(
        writer: StreamWriter<u8>,
        reader: StreamReader<u8>,
        finish_future: Option<FutureReader<Result<(), ErrorCode>>>,
    ) -> Self {
        Self {
            writer,
            reader,
            finish_future,
        }
    }

    pub async fn connect_wasi(
        host: &str,
        port: u16,
        _timeout: Option<Duration>,
        tls_parameters: Option<TlsParameters>,
    ) -> Result<Self, Error> {
        let addresses = wasip3::sockets::ip_name_lookup::resolve_addresses(host.to_string())
            .await
            .map_err(|e| {
                error::connection(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("DNS failed: {:?}", e),
                ))
            })?;

        let ip = addresses.into_iter().next().ok_or_else(|| {
            error::connection(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No addresses found",
            ))
        })?;

        let addr = match ip {
            wasip3::sockets::types::IpAddress::Ipv4(ipv4) => {
                IpSocketAddress::Ipv4(wasip3::sockets::types::Ipv4SocketAddress {
                    address: ipv4,
                    port,
                })
            }
            wasip3::sockets::types::IpAddress::Ipv6(ipv6) => {
                IpSocketAddress::Ipv6(wasip3::sockets::types::Ipv6SocketAddress {
                    address: ipv6,
                    port,
                    flow_info: 0,
                    scope_id: 0,
                })
            }
        };

        let socket = wasip3::sockets::types::TcpSocket::create(
            wasip3::sockets::types::IpAddressFamily::Ipv4,
        )
        .expect("failed to create TCP socket");
        eprintln!(
            "wasip3: created tcp socket for addr {:?} port {}",
            addr, port
        );

        socket.connect(addr).await.map_err(|e| {
            error::connection(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Connection failed: {:?}", e),
            ))
        })?;
        eprintln!("wasip3: Socket connection successful");

        // Create the guest-side writer (data_tx) and the host-side receiver (data_rx)
        let (data_tx, data_rx) = wit_stream::new();
        eprintln!("wasip3: Stream created");

        // CRITICAL: Get the reader FIRST before spawning the send task
        // socket.receive() is non-blocking - it just sets up the receive channel
        let (incoming_reader, finish_future) = socket.receive();
        eprintln!("wasip3: incoming reader fetched");

        // Now spawn the send task to run in the background
        // This keeps the write channel open and forwards data from data_tx to the socket
        // The spawned task will run concurrently and complete after the export returns
        spawn(async move {
            match socket.send(data_rx).await {
                Ok(_) => eprintln!("wasip3: send task completed successfully"),
                Err(e) => eprintln!("wasip3: send task failed: {:?}", e),
            }
        });

        eprintln!("wasip3: send task spawned, returning stream");

        if let Some(tls_parameters) = tls_parameters {
            // stream.upgrade_tls(tls_parameters).await?;
            //
            // Wasi-specific tls upgrade should take place here
            // Stub for now, p3 support shaky
            //
            // TODO!
        }

        Ok(Self::from_halves(
            data_tx,
            incoming_reader,
            Some(finish_future),
        ))
    }
}
