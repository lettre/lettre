//! Partial parsers implementation of [RFC3629]: UTF-8, a
//! transformation format of ISO 10646.
//!
//! [RFC3629]: https://datatracker.ietf.org/doc/html/rfc3629#section-4

use chumsky::prelude::*;
use once_cell::sync::Lazy;

const UTF8_E0: char = 0xE0 as char;
const UTF8_ED: char = 0xED as char;
const UTF8_F0: char = 0xF0 as char;
const UTF8_F4: char = 0xF4 as char;

static UTF8_80_8F: Lazy<Vec<char>> = Lazy::new(|| ((0x80 as char)..=(0x8F as char)).collect());
static UTF8_80_9F: Lazy<Vec<char>> = Lazy::new(|| ((0x80 as char)..=(0x9F as char)).collect());
static UTF8_80_BF: Lazy<Vec<char>> = Lazy::new(|| ((0x80 as char)..=(0xBF as char)).collect());
static UTF8_90_BF: Lazy<Vec<char>> = Lazy::new(|| ((0x90 as char)..=(0xBF as char)).collect());
static UTF8_A0_BF: Lazy<Vec<char>> = Lazy::new(|| ((0xA0 as char)..=(0xBF as char)).collect());
static UTF8_C2_DF: Lazy<Vec<char>> = Lazy::new(|| ((0xC2 as char)..=(0xDF as char)).collect());
static UTF8_E1_EC: Lazy<Vec<char>> = Lazy::new(|| ((0xE1 as char)..=(0xEC as char)).collect());
static UTF8_EE_EF: Lazy<Vec<char>> = Lazy::new(|| ((0xEE as char)..=(0xEF as char)).collect());
static UTF8_F1_F3: Lazy<Vec<char>> = Lazy::new(|| ((0xF1 as char)..=(0xF3 as char)).collect());

// 4.  Syntax of UTF-8 Byte Sequences
// https://datatracker.ietf.org/doc/html/rfc3629#section-4

// UTF8-2      = %xC2-DF UTF8-tail
pub(super) fn utf8_2() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    one_of(UTF8_C2_DF.as_slice()).chain(utf8_tail())
}

// UTF8-3      = %xE0 %xA0-BF UTF8-tail / %xE1-EC 2( UTF8-tail ) /
//               %xED %x80-9F UTF8-tail / %xEE-EF 2( UTF8-tail )
pub(super) fn utf8_3() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((
        just(UTF8_E0)
            .chain(one_of(UTF8_A0_BF.as_slice()))
            .chain(utf8_tail()),
        one_of(UTF8_E1_EC.as_slice()).chain(utf8_tail().repeated().exactly(2)),
        just(UTF8_ED)
            .chain(one_of(UTF8_80_9F.as_slice()))
            .chain(utf8_tail()),
        one_of(UTF8_EE_EF.as_slice()).chain(utf8_tail().repeated().exactly(2)),
    ))
}

// UTF8-4      = %xF0 %x90-BF 2( UTF8-tail ) / %xF1-F3 3( UTF8-tail ) /
//               %xF4 %x80-8F 2( UTF8-tail )
pub(super) fn utf8_4() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((
        just(UTF8_F0)
            .chain(one_of(UTF8_90_BF.as_slice()))
            .chain(utf8_tail().repeated().exactly(2)),
        one_of(UTF8_F1_F3.as_slice()).chain(utf8_tail().repeated().exactly(3)),
        just(UTF8_F4)
            .chain(one_of(UTF8_80_8F.as_slice()))
            .chain(utf8_tail().repeated().exactly(2)),
    ))
}

// UTF8-tail   = %x80-BF
pub(super) fn utf8_tail() -> impl Parser<char, char, Error = Simple<char>> {
    one_of(UTF8_80_BF.as_slice())
}
