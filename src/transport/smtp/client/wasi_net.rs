//! WASI-compatible async network stream for lettre (wasip3)
//! Implements futures_io::AsyncRead/AsyncWrite using wit_stream halves

//! WASI-compatible async network stream for lettre (wasip3)
//! Concrete types for the wit-bindgen generated stream halves.

//#![cfg(target_arch = "wasm32")]

use std::time::Duration;

use crate::transport::smtp::{error, Error};
use wasip3::sockets::types::IpSocketAddress;
use wasip3::wit_bindgen::rt::async_support::{FutureReader, StreamReader, StreamWriter};
use wasip3::{sockets::types::ErrorCode, wit_stream};

/// WASI-compatible network stream for lettre — concrete halves from the host
pub struct WasiNetworkStream {
    /// Component -> Host writer (guest writes bytes here)
    pub writer: StreamWriter<u8>,
    /// Host -> Component reader (guest reads bytes from here)
    pub reader: StreamReader<u8>,
    /// Optional future returned by `TcpSocket::receive()` that the guest should
    /// await or drive to completion when closing the connection. It resolves to
    /// Result<(), ErrorCode> (host socket error code) so keep that concrete type.
    pub finish_future: Option<FutureReader<Result<(), ErrorCode>>>,
}

impl WasiNetworkStream {
    /// Create a new `WasiNetworkStream` from the two halves returned by the
    /// host/`wit_stream::new()` and the optional finish future returned by the
    /// socket receive API.
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
        timeout: Option<Duration>,
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
        eprintln!("wasip3: created tcp socket for port {}", port);

        // Connect with timeout handling
        let connect_result = if let Some(timeout_duration) = timeout {
            // Wrap the connect operation with a timeout
            async fn connect_with_timeout(
                socket: &wasip3::sockets::types::TcpSocket,
                addr: IpSocketAddress,
                timeout: Duration,
            ) -> Result<(), ErrorCode> {
                // Compiling to wasm32 target should resolve this error, but compiler will complain for now.
                match tokio1_crate::time::timeout(timeout, socket.connect(addr)).await {
                    Ok(result) => result,
                    Err(_) => Err(ErrorCode::Timeout),
                }
            }

            connect_with_timeout(&socket, addr, timeout_duration).await
        } else {
            socket.connect(addr).await
        };

        // Handle connection errors, including timeout
        connect_result.map_err(|e| {
            error::connection(std::io::Error::new(
                match e {
                    ErrorCode::Timeout => std::io::ErrorKind::TimedOut,
                    ErrorCode::ConnectionRefused => std::io::ErrorKind::ConnectionRefused,
                    ErrorCode::ConnectionReset => std::io::ErrorKind::ConnectionReset,
                    ErrorCode::ConnectionAborted => std::io::ErrorKind::ConnectionAborted,
                    _ => std::io::ErrorKind::Other,
                },
                format!("Connection failed: {:?}", e),
            ))
        })?;

        // Create the guest-side writer (data_tx) and the host-side receiver (data_rx)
        let (data_tx, data_rx) = wit_stream::new();
        // Give the host the receiver so it can forward writes from the guest to the socket
        socket.send(data_rx).await.map_err(error::connection)?;
        // Get the incoming reader and the finish future
        let (incoming_reader, finish_future) = socket.receive();

        Ok(Self::from_halves(
            data_tx,
            incoming_reader,
            Some(finish_future),
        ))
    }
}
