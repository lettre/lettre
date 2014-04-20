/*!
 * Common definitions for SMTP
 *
 * Needs to be organized later.
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
    match (addr.slice_to(1), addr.slice_from(addr.len()-1)) {
        ("<", ">") => addr.to_owned(),
        _          => format!("<{:s}>", addr)
    }
}

/// Remove quotes from emails
pub fn unquote_email_address(addr: &str) -> ~str {
    match (addr.slice_to(1), addr.slice_from(addr.len() - 1)) {
        ("<", ">") => addr.slice(1, addr.len() - 1).to_owned(),
        _          => addr.to_owned()
    }
}

/// Returns the first word of a string, or the string if it contains no space
pub fn get_first_word(string: &str) -> ~str {
    string.split_str(CRLF).next().unwrap().splitn(' ', 1).next().unwrap().to_owned()
}

#[cfg(test)]
mod test {
    #[test]
    fn test_quote_email_address() {
        assert!(super::quote_email_address("plop") == ~"<plop>");
        assert!(super::quote_email_address("<plop>") == ~"<plop>");
    }

    #[test]
    fn test_unquote_email_address() {
        assert!(super::unquote_email_address("<plop>") == ~"plop");
        assert!(super::unquote_email_address("plop") == ~"plop");
    }

    #[test]
    fn test_get_first_word() {
        assert!(super::get_first_word("first word") == ~"first");
        assert!(super::get_first_word("first word\ntest") == ~"first");
        assert!(super::get_first_word("first") == ~"first");
    }
}
