//! This transport creates a file for each email, containing the enveloppe information and the email
//! itself.

use transport::EmailTransport;
use transport::error::EmailResult;
use transport::smtp::response::Response;
use transport::smtp::response::{Category, Code, Severity};
use email::SendableEmail;

/// Writes the content and the enveloppe information to a file
pub struct SendmailEmailTransport {
    command: String,
}

impl SendmailEmailTransport {
    /// Creates a new transport to the default "sendmail" command
    pub fn new() -> SendmailEmailTransport {
        SendmailEmailTransport { command: "sendmail".to_string() }
    }

    /// Creates a new transport with a custom sendmail command
    pub fn new_with_command(command: &str) -> SendmailEmailTransport {
        SendmailEmailTransport { command: command.to_string() }
    }
}

impl EmailTransport for SendmailEmailTransport {
    fn send<T: SendableEmail>(&mut self, email: T) -> EmailResult {


        // Build TO list
        // Set FROM
        // Send content

        let sendmail_sh_frist_half = "sendmail ".to_string() + &to_address;
        let sendmail_sh_second_half = " < email.txt".to_string();
        let sendmail_sh = sendmail_sh_frist_half + &sendmail_sh_second_half;

        let output = Command::new(self.command)
                         .arg("-c")
                         .arg(sendmail_sh)
                         .output()
                         .unwrap_or_else(|e| panic!("failed to execute process: {}", e));

        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));




        let mut file = self.path.clone();
        file.push(format!("{}.txt", email.message_id()));

        let mut f = try!(File::create(file.as_path()));

        let log_line = format!("{}: from=<{}> to=<{}>\n",
                               email.message_id(),
                               email.from_address(),
                               email.to_addresses().join("> to=<"));

        try!(f.write_all(log_line.as_bytes()));
        try!(f.write_all(format!("{}", email.message()).as_bytes()));

        info!("{} status=<written>", log_line);

        Ok(Response::new(Code::new(Severity::PositiveCompletion, Category::MailSystem, 0),
                         vec![format!("Ok: email written to {}",
                                      file.to_str().unwrap_or("non-UTF-8 path"))]))
    }

    fn close(&mut self) {
        ()
    }
}
