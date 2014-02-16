#[crate_id = "client"];

extern crate smtp;
use std::io::net::tcp::TcpStream;
use smtp::client::SmtpClient;

fn main() {
    let mut email_client: SmtpClient<TcpStream> = SmtpClient::new("localhost", None, None);
    email_client.send_mail("amousset@localhost", [&"amousset@localhost"], "Test email");
}
