//! Partial parsers implementation of [RFC2822]: Internet Message
//! Format.
//!
//! [RFC2822]: https://datatracker.ietf.org/doc/html/rfc2822

use chumsky::{error::Cheap, prelude::*};

use super::{rfc2234, rfc5336};

// 3.2.1. Primitive Tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.1

// NO-WS-CTL       =       %d1-8 /         ; US-ASCII control characters
//                         %d11 /          ;  that do not include the
//                         %d12 /          ;  carriage return, line feed,
//                         %d14-31 /       ;  and white space characters
//                         %d127
fn no_ws_ctl() -> impl Parser<char, char, Error = Cheap<char>> {
    filter(|c| matches!(u32::from(*c), 1..=8 | 11 | 12 | 14..=31 | 127))
}

// text            =       %d1-9 /         ; Characters excluding CR and LF
//                         %d11 /
//                         %d12 /
//                         %d14-127 /
//                         obs-text
fn text() -> impl Parser<char, char, Error = Cheap<char>> {
    filter(|c| matches!(u32::from(*c), 1..=9 | 11 | 12 | 14..=127))
}

// 3.2.2. Quoted characters
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.2

// quoted-pair     =       ("\" text) / obs-qp
fn quoted_pair() -> impl Parser<char, char, Error = Cheap<char>> {
    just('\\').ignore_then(text())
}

// 3.2.3. Folding white space and comments
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.3

// FWS             =       ([*WSP CRLF] 1*WSP) /   ; Folding white space
//                         obs-FWS
pub(super) fn fws() -> impl Parser<char, Option<char>, Error = Cheap<char>> {
    rfc2234::wsp()
        .or_not()
        .then_ignore(rfc2234::wsp().ignored().repeated())
}

// CFWS            =       *([FWS] comment) (([FWS] comment) / FWS)
pub(super) fn cfws() -> impl Parser<char, Option<char>, Error = Cheap<char>> {
    // TODO: comment are not currently supported, so for now a cfws is
    // the same as a fws.
    fws()
}

// 3.2.4. Atom
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.4

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
pub(super) fn atext() -> impl Parser<char, char, Error = Cheap<char>> {
    choice((
        rfc2234::alpha(),
        rfc2234::digit(),
        filter(|c| {
            matches!(
                *c,
                '!' | '#'
                    | '$'
                    | '%'
                    | '&'
                    | '\''
                    | '*'
                    | '+'
                    | '-'
                    | '/'
                    | '='
                    | '?'
                    | '^'
                    | '_'
                    | '`'
                    | '{'
                    | '|'
                    | '}'
                    | '~'
            )
        }),
        // also allow non ASCII UTF8 chars
        rfc5336::utf8_non_ascii(),
    ))
}

// atom            =       [CFWS] 1*atext [CFWS]
pub(super) fn atom() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    cfws().chain(atext().repeated().at_least(1))
}

// dot-atom        =       [CFWS] dot-atom-text [CFWS]
pub(super) fn dot_atom() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    cfws().chain(dot_atom_text())
}

// dot-atom-text   =       1*atext *("." 1*atext)
pub(super) fn dot_atom_text() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    atext().repeated().at_least(1).chain(
        just('.')
            .chain(atext().repeated().at_least(1))
            .repeated()
            .at_least(1)
            .flatten(),
    )
}

// 3.2.5. Quoted strings
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.5

// qtext           =       NO-WS-CTL /     ; Non white space controls
//
//                         %d33 /          ; The rest of the US-ASCII
//                         %d35-91 /       ;  characters not including "\"
//                         %d93-126        ;  or the quote character
fn qtext() -> impl Parser<char, char, Error = Cheap<char>> {
    choice((
        filter(|c| matches!(u32::from(*c), 33 | 35..=91 | 93..=126)),
        no_ws_ctl(),
    ))
}

// qcontent        =       qtext / quoted-pair
pub(super) fn qcontent() -> impl Parser<char, char, Error = Cheap<char>> {
    choice((qtext(), quoted_pair(), rfc5336::utf8_non_ascii()))
}

// quoted-string   =       [CFWS]
//                         DQUOTE *([FWS] qcontent) [FWS] DQUOTE
//                         [CFWS]
fn quoted_string() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    rfc2234::dquote()
        .ignore_then(fws().chain(qcontent()).repeated().flatten())
        .then_ignore(text::whitespace())
        .then_ignore(rfc2234::dquote())
}

// 3.2.6. Miscellaneous tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.6

// word            =       atom / quoted-string
fn word() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    choice((quoted_string(), atom()))
}

// phrase          =       1*word / obs-phrase
fn phrase() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    choice((obs_phrase(), word().repeated().at_least(1).flatten()))
}

// 3.4. Address Specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4

// mailbox         =       name-addr / addr-spec
pub(crate) fn mailbox() -> impl Parser<char, (Option<String>, (String, String)), Error = Cheap<char>>
{
    choice((name_addr(), addr_spec().map(|addr| (None, addr))))
        .padded()
        .then_ignore(end())
}

// name-addr       =       [display-name] angle-addr
fn name_addr() -> impl Parser<char, (Option<String>, (String, String)), Error = Cheap<char>> {
    display_name().collect().or_not().then(angle_addr())
}

// angle-addr      =       [CFWS] "<" addr-spec ">" [CFWS] / obs-angle-addr
fn angle_addr() -> impl Parser<char, (String, String), Error = Cheap<char>> {
    addr_spec()
        .delimited_by(just('<').ignored(), just('>').ignored())
        .padded()
}

// display-name    =       phrase
fn display_name() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    phrase()
}

// mailbox-list    =       (mailbox *("," mailbox)) / obs-mbox-list
pub(crate) fn mailbox_list(
) -> impl Parser<char, Vec<(Option<String>, (String, String))>, Error = Cheap<char>> {
    choice((name_addr(), addr_spec().map(|addr| (None, addr))))
        .separated_by(just(',').padded())
        .then_ignore(end())
}

// 3.4.1. Addr-spec specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4.1

// addr-spec       =       local-part "@" domain
pub(super) fn addr_spec() -> impl Parser<char, (String, String), Error = Cheap<char>> {
    local_part()
        .collect()
        .then_ignore(just('@'))
        .then(domain().collect())
}

// local-part      =       dot-atom / quoted-string / obs-local-part
pub(super) fn local_part() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    choice((dot_atom(), quoted_string(), obs_local_part()))
}

// domain          =       dot-atom / domain-literal / obs-domain
pub(super) fn domain() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    // NOTE: omitting domain-literal since it may never be used
    choice((dot_atom(), obs_domain()))
}

// 4.1. Miscellaneous obsolete tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-4.1

// obs-phrase      =       word *(word / "." / CFWS)
fn obs_phrase() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    // NOTE: the CFWS is already captured by the word, no need to add
    // it there.
    word().chain(
        choice((word(), just('.').repeated().exactly(1)))
            .repeated()
            .flatten(),
    )
}

// 4.4. Obsolete Addressing
// https://datatracker.ietf.org/doc/html/rfc2822#section-4.4

// obs-local-part  =       word *("." word)
pub(super) fn obs_local_part() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    word().chain(just('.').chain(word()).repeated().flatten())
}

// obs-domain      =       atom *("." atom)
pub(super) fn obs_domain() -> impl Parser<char, Vec<char>, Error = Cheap<char>> {
    atom().chain(just('.').chain(atom()).repeated().flatten())
}
