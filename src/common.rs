// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Constants defined in SMTP RFCs

#![unstable]

use std::io::net::ip::Port;

/// Default smtp port
pub static SMTP_PORT: Port = 25;

/// Default smtps port
pub static SMTPS_PORT: Port = 465;

/// Default submission port
pub static SUBMISSION_PORT: Port = 587;

/// The word separator for SMTP transactions
pub static SP: &'static str = " ";

/// The line ending for SMTP transactions (carriage return + line feed)
pub static CRLF: &'static str = "\r\n";

/// Carriage return
pub static CR: &'static str = "\r";

/// Line feed
pub static LF: &'static str = "\n";

/// Colon
pub static COLON: &'static str = ":";

/// The ending of message content
pub static MESSAGE_ENDING: &'static str = "\r\n.\r\n";
