//! Partial parsers implementation of [RFC2234]: Augmented BNF for
//! Syntax Specifications: ABNF.
//!
//! [RFC2234]: https://datatracker.ietf.org/doc/html/rfc2234

use chumsky::{error::Cheap, prelude::*};

// 6.1  Core Rules
// https://datatracker.ietf.org/doc/html/rfc2234#section-6.1

// ALPHA          =  %x41-5A / %x61-7A   ; A-Z / a-z
pub(super) fn alpha() -> impl Parser<char, char, Error = Cheap<char>> {
    filter(|c: &char| matches!(*c, 'A'..='Z' | 'a'..='z'))
}

// CRLF           =  CR LF
//                        ; Internet standard newline
pub(super) fn crlf() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    just('\r').chain(just('\n'))
}

// DIGIT          =  %x30-39
//                        ; 0-9
pub(super) fn digit() -> impl Parser<char, char, Error = Cheap<char>> {
    filter(|c: &char| matches!(*c, '0'..='9'))
}

// DQUOTE         =  %x22
//                        ; " (Double Quote)
pub(super) fn dquote() -> impl Parser<char, char, Error = Cheap<char>> {
    just('"')
}

// WSP            =  SP / HTAB
//                        ; white space
pub(super) fn wsp() -> impl Parser<char, char, Error = Cheap<char>> {
    choice((just(' '), just('\t')))
}
