//! Partial parsers implementation of [RFC2821]: Simple Mail Transfer
//! Protocol.
//!
//! [RFC2821]: https://datatracker.ietf.org/doc/html/rfc2821

use chumsky::prelude::*;

use super::rfc2234;

// 4.1.3 Address Literals
// https://datatracker.ietf.org/doc/html/rfc2821#section-4.1.3

// Let-dig = ALPHA / DIGIT
pub(super) fn let_dig() -> impl Parser<char, char, Error = Simple<char>> {
    choice((rfc2234::alpha(), rfc2234::digit()))
}
