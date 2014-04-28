// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*! SMTP library
 
This library implements a simple SMTP client.
RFC 5321 : https://tools.ietf.org/html/rfc5321#section-4.1

It does NOT manages email content.

It also implements the following extesnions
    8BITMIME (RFC 6152 : https://tools.ietf.org/html/rfc6152)

# Usage

```
let mut email_client: SmtpClient<StrBuf, TcpStream> = SmtpClient::new(StrBuf::from_str("localhost"), None, None);
email_client.send_mail(StrBuf::from_str("<user@example.com>"), vec!(StrBuf::from_str("<user@example.org>")), StrBuf::from_str("Test email"));
```

# TODO:
    Add SSL/TLS
    Add AUTH

*/

#![crate_id = "smtp#0.1-pre"]

#![desc = "Rust SMTP client"]
#![comment = "Simple SMTP client"]
#![license = "ASL2"]
#![crate_type = "lib"]

#![doc(html_root_url = "http://www.rust-ci.org/amousset/rust-smtp/doc/")]

#![feature(macro_rules)]
#![deny(non_camel_case_types)]
#![deny(missing_doc)]

#![feature(phase)]
#[phase(syntax, link)] extern crate log;

pub mod commands;
pub mod common;
pub mod client;
