/*! 
 * Common definitions for SMTP
 */

use std::io::net::ip::Port;

/// Default SMTP port
pub static SMTP_PORT: Port = 25;
//pub static SMTPS_PORT: Port = 465;
//pub static SUBMISSION_PORT: Port = 587;

/// End of SMTP commands
pub static CRLF: &'static str = "\r\n";

/// Add quotes to emails
pub fn quote_email_address(addr: &str) -> ~str {
    return format!("<{:s}>", addr).to_owned();
}
