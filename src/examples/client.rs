#[crate_id = "client"];

extern mod smtp;
use smtp::client::SmtpClient;

fn main() {
    let mut email_client: SmtpClient = SmtpClient::new("localhost", None, None);
    email_client.send_mail("moostik@minet.net", [&"moostik@localhost"], "plop");
}