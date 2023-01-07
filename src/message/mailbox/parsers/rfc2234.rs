//! Partial parsers implementation of [RFC2234]: Augmented BNF for
//! Syntax Specifications: ABNF.
//!
//! [RFC2234]: https://datatracker.ietf.org/doc/html/rfc2234

use chumsky::prelude::*;
use once_cell::sync::Lazy;

const DQUOTE: char = 0x22 as char;

static US_ASCII_30_39: Lazy<Vec<char>> = Lazy::new(|| ((0x30 as char)..=(0x39 as char)).collect());
static US_ASCII_41_5A: Lazy<Vec<char>> = Lazy::new(|| ((0x41 as char)..=(0x5A as char)).collect());
static US_ASCII_61_7A: Lazy<Vec<char>> = Lazy::new(|| ((0x61 as char)..=(0x7A as char)).collect());

// 6.1  Core Rules
// https://datatracker.ietf.org/doc/html/rfc2234#section-6.1

// ALPHA          =  %x41-5A / %x61-7A   ; A-Z / a-z
pub(super) fn alpha() -> impl Parser<char, char, Error = Simple<char>> {
    choice((
        one_of(US_ASCII_41_5A.as_slice()),
        one_of(US_ASCII_61_7A.as_slice()),
    ))
}

// DIGIT          =  %x30-39
//                        ; 0-9
pub(super) fn digit() -> impl Parser<char, char, Error = Simple<char>> {
    one_of(US_ASCII_30_39.as_slice())
}

// DQUOTE         =  %x22
//                        ; " (Double Quote)
pub(super) fn dquote() -> impl Parser<char, char, Error = Simple<char>> {
    just(DQUOTE)
}
