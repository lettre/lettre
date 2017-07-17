extern crate lettre;

mod transport_smtp;
mod transport_sendmail;
mod transport_stub;
#[cfg(feature = "file-transport")]
mod transport_file;
