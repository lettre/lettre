//! Partial parsers implementation of [RFC2234]: Augmented BNF for
//! Syntax Specifications: ABNF.
//!
//! [RFC2234]: https://datatracker.ietf.org/doc/html/rfc2234

use chumsky::prelude::*;

const DQUOTE: char = 0x22 as char;

// 6.1  Core Rules
// https://datatracker.ietf.org/doc/html/rfc2234#section-6.1

// ALPHA          =  %x41-5A / %x61-7A   ; A-Z / a-z
pub(super) fn alpha() -> impl Parser<char, char, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_alphabetic())
}

// DIGIT          =  %x30-39
//                        ; 0-9
pub(super) fn digit() -> impl Parser<char, char, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_digit())
}

// DQUOTE         =  %x22
//                        ; " (Double Quote)
pub(super) fn dquote() -> impl Parser<char, char, Error = Simple<char>> {
    just(DQUOTE)
}
