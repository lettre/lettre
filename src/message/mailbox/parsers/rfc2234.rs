//! Partial parsers implementation of [RFC2234]: Augmented BNF for
//! Syntax Specifications: ABNF.
//!
//! [RFC2234]: https://datatracker.ietf.org/doc/html/rfc2234

use nom::{
    branch::alt,
    character::complete::{char, satisfy},
    IResult, Parser,
};

// 6.1  Core Rules
// https://datatracker.ietf.org/doc/html/rfc2234#section-6.1

// ALPHA          =  %x41-5A / %x61-7A   ; A-Z / a-z
pub(super) fn alpha(input: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_alphabetic()).parse(input)
}

// DIGIT          =  %x30-39
//                        ; 0-9
pub(super) fn digit(input: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_digit()).parse(input)
}

// DQUOTE         =  %x22
//                        ; " (Double Quote)
pub(super) fn dquote(input: &str) -> IResult<&str, char> {
    char('"').parse(input)
}

// WSP            =  SP / HTAB
//                        ; white space
pub(super) fn wsp(input: &str) -> IResult<&str, char> {
    alt((char(' '), char('\t'))).parse(input)
}
