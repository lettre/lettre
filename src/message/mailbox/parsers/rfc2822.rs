//! Partial parsers implementation of [RFC2822]: Internet Message
//! Format.
//!
//! [RFC2822]: https://datatracker.ietf.org/doc/html/rfc2822

use chumsky::prelude::*;
use once_cell::sync::Lazy;

use super::{rfc2234, rfc5336};

// 3.2.1. Primitive Tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.1

static TEXT: Lazy<Vec<char>> = Lazy::new(|| {
    let mut range: Vec<char> = vec![];
    range.extend((1 as char)..=(9 as char));
    range.push(11 as char);
    range.push(12 as char);
    range.extend((14 as char)..=(127 as char));
    range
});

// text            =       %d1-9 /         ; Characters excluding CR and LF
//                         %d11 /
//                         %d12 /
//                         %d14-127 /
//                         obs-text
fn text() -> impl Parser<char, char, Error = Simple<char>> {
    one_of(TEXT.as_slice())
}

// 3.2.2. Quoted characters
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.2

// quoted-pair     =       ("\" text) / obs-qp
fn quoted_pair() -> impl Parser<char, char, Error = Simple<char>> {
    just('\\').ignore_then(text())
}

// 3.2.4. Atom
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.4

static ATEXT: Lazy<Vec<char>> = Lazy::new(|| {
    let mut range: Vec<char> = vec![];
    range.extend('0'..='9');
    range.extend('a'..='z');
    range.extend('A'..='Z');
    range.extend("!#$%&\'*+-/=?^_`{|}~".chars());
    range
});

// atext           =       ALPHA / DIGIT / ; Any character except controls,
//                         "!" / "#" /     ;  SP, and specials.
//                         "$" / "%" /     ;  Used for atoms
//                         "&" / "'" /
//                         "*" / "+" /
//                         "-" / "/" /
//                         "=" / "?" /
//                         "^" / "_" /
//                         "`" / "{" /
//                         "|" / "}" /
//                         "~"
pub(super) fn atext() -> impl Parser<char, char, Error = Simple<char>> {
    one_of(ATEXT.as_slice())
}

// 3.2.5. Quoted strings
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.5

static QTEXT: Lazy<Vec<char>> = Lazy::new(|| {
    let mut range: Vec<char> = vec![33 as char];
    range.extend((35 as char)..=(91 as char));
    range.extend((93 as char)..=(126 as char));
    range
});

// qtext           =       NO-WS-CTL /     ; Non white space controls
//
//                         %d33 /          ; The rest of the US-ASCII
//                         %d35-91 /       ;  characters not including "\"
//                         %d93-126        ;  or the quote character
fn qtext() -> impl Parser<char, char, Error = Simple<char>> {
    one_of(QTEXT.as_slice())
}

// qcontent        =       qtext / quoted-pair
pub(super) fn qcontent() -> impl Parser<char, char, Error = Simple<char>> {
    choice((qtext(), quoted_pair()))
}

// 3.4. Address Specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4

// mailbox         =       name-addr / addr-spec
pub(crate) fn mailbox(
) -> impl Parser<char, (Option<String>, (String, String)), Error = Simple<char>> {
    choice((addr_spec().map(|addr| (None, addr)), name_addr()))
}

// name-addr       =       [display-name] angle-addr
fn name_addr() -> impl Parser<char, (Option<String>, (String, String)), Error = Simple<char>> {
    rfc2234::dquote()
        .or_not()
        .ignore_then(
            // NOTE: take everything available between potential
            // quotes in order to make the parsing of the display-name
            // the most flexible possible
            take_until(rfc2234::dquote().or_not().ignore_then(angle_addr())).map(
                |(display_name, address)| {
                    (
                        if display_name.is_empty() {
                            None
                        } else {
                            Some(String::from_iter(display_name))
                        },
                        address,
                    )
                },
            ),
        )
        .padded()
}

// angle-addr      =       [CFWS] "<" addr-spec ">" [CFWS] / obs-angle-addr
fn angle_addr() -> impl Parser<char, (String, String), Error = Simple<char>> {
    addr_spec()
        .delimited_by(just('<').ignored(), just('>').ignored())
        .padded()
}

// mailbox-list    =       (mailbox *("," mailbox)) / obs-mbox-list
pub(crate) fn mailbox_list(
) -> impl Parser<char, Vec<(Option<String>, (String, String))>, Error = Simple<char>> {
    mailbox().chain(just(',').ignore_then(mailbox()).repeated())
}

// 3.4.1. Addr-spec specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4.1

// addr-spec       =       local-part "@" domain
fn addr_spec() -> impl Parser<char, (String, String), Error = Simple<char>> {
    // NOTE: we use the rfc5336 unicode mailbox spec instead of the
    // rfc2822 address spec because in `lettre` context headers are
    // already pre-parsed and decoded.
    rfc5336::u_mailbox()
}
