use std::{env, process, time::Duration};

use lettre::SmtpTransport;

fn main() {
    tracing_subscriber::fmt::init();

    let smtp_host = match env::args().nth(1) {
        Some(smtp_host) => smtp_host,
        None => {
            println!("Please provide the SMTP host as the first argument to this command");
            process::exit(1);
        }
    };

    // TLS wrapped connection
    {
        tracing::info!(
            "Trying to establish a TLS wrapped connection to {}",
            smtp_host
        );

        let transport = SmtpTransport::relay(&smtp_host)
            .expect("build SmtpTransport::relay")
            .timeout(Some(Duration::from_secs(10)))
            .build();
        match transport.test_connection() {
            Ok(true) => {
                tracing::info!("Successfully connected to {} via a TLS wrapped connection (SmtpTransport::relay). This is the fastest option available for connecting to an SMTP server", smtp_host);
            }
            Ok(false) => {
                tracing::error!("Couldn't connect to {} via a TLS wrapped connection. No more information is available", smtp_host);
            }
            Err(err) => {
                tracing::error!(err = %err, "Couldn't connect to {} via a TLS wrapped connection", smtp_host);
            }
        }
    }

    println!();

    // Plaintext connection which MUST then successfully upgrade to TLS via STARTTLS
    {
        tracing::info!("Trying to establish a plaintext connection to {} and then upgrading it via the SMTP STARTTLS extension", smtp_host);

        let transport = SmtpTransport::starttls_relay(&smtp_host)
            .expect("build SmtpTransport::starttls_relay")
            .timeout(Some(Duration::from_secs(10)))
            .build();
        match transport.test_connection() {
            Ok(true) => {
                tracing::info!("Successfully connected to {} via a plaintext connection which then got upgraded to TLS via the SMTP STARTTLS extension (SmtpTransport::starttls_relay). This is the second best option after the previous TLS wrapped option", smtp_host);
            }
            Ok(false) => {
                tracing::error!(
                    "Couldn't connect to {} via STARTTLS. No more information is available",
                    smtp_host
                );
            }
            Err(err) => {
                tracing::error!(err = %err, "Couldn't connect to {} via STARTTLS", smtp_host);
            }
        }
    }

    println!();

    // Plaintext connection (very insecure)
    {
        tracing::info!(
            "Trying to establish a plaintext connection to {}",
            smtp_host
        );

        let transport = SmtpTransport::builder_dangerous(&smtp_host)
            .timeout(Some(Duration::from_secs(10)))
            .build();
        match transport.test_connection() {
            Ok(true) => {
                tracing::info!("Successfully connected to {} via a plaintext connection. This option is very insecure and shouldn't be used on the public internet (SmtpTransport::builder_dangerous)", smtp_host);
            }
            Ok(false) => {
                tracing::error!(
                    "Couldn't connect to {} via a plaintext connection. No more information is available",
                    smtp_host
                );
            }
            Err(err) => {
                tracing::error!(err = %err, "Couldn't connect to {} via a plaintext connection", smtp_host);
            }
        }
    }
}
