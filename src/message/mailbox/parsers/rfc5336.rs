//! Partial parsers implementation of [RFC5336]: SMTP Extension for
//! Internationalized Email Addresses.
//!
//! [RFC5336]: https://datatracker.ietf.org/doc/html/rfc5336

use chumsky::{error::Cheap, prelude::*};

// 3.3.  Extended Mailbox Address Syntax
// https://datatracker.ietf.org/doc/html/rfc5336#section-3.3

// UTF8-non-ascii = UTF8-2 / UTF8-3 / UTF8-4
// UTF8-2 =  <See Section 4 of RFC 3629>
// UTF8-3 =  <See Section 4 of RFC 3629>
// UTF8-4 =  <See Section 4 of RFC 3629>
pub(super) fn utf8_non_ascii() -> impl Parser<char, char, Error = Cheap<char>> {
    filter(|c: &char| c.len_utf8() > 1)
}
