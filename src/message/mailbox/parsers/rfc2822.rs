//! Partial parsers implementation of [RFC2822]: Internet Message
//! Format.
//!
//! [RFC2822]: https://datatracker.ietf.org/doc/html/rfc2822

use chumsky::prelude::*;

use super::{rfc2234, rfc5336};

// 3.2.1. Primitive Tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.1

// NO-WS-CTL       =       %d1-8 /         ; US-ASCII control characters
//                         %d11 /          ;  that do not include the
//                         %d12 /          ;  carriage return, line feed,
//                         %d14-31 /       ;  and white space characters
//                         %d127
fn no_ws_ctl() -> impl Parser<char, char, Error = Simple<char>> {
    filter(|c| matches!(u32::from(*c), 1..=8 | 11 | 12 | 14..=31 | 127))
}

// text            =       %d1-9 /         ; Characters excluding CR and LF
//                         %d11 /
//                         %d12 /
//                         %d14-127 /
//                         obs-text
fn text() -> impl Parser<char, char, Error = Simple<char>> {
    filter(|c| matches!(u32::from(*c), 1..=9 | 11 | 12 | 14..=127))
}

// 3.2.2. Quoted characters
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.2

// quoted-pair     =       ("\" text) / obs-qp
fn quoted_pair() -> impl Parser<char, char, Error = Simple<char>> {
    just('\\').ignore_then(text())
}

// 3.2.3. Folding white space and comments
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.3

// FWS             =       ([*WSP CRLF] 1*WSP) /   ; Folding white space
//                         obs-FWS
pub fn fws() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    // NOTE: obs-FWS leads to recursion, skipping it
    rfc2234::wsp()
        .repeated()
        .chain(rfc2234::crlf())
        .or_not()
        .flatten()
        .chain(rfc2234::wsp().repeated().at_least(1))
}

// ctext           =       NO-WS-CTL /     ; Non white space controls
//
//                         %d33-39 /       ; The rest of the US-ASCII
//                         %d42-91 /       ;  characters not including "(",
//                         %d93-126        ;  ")", or "\"
pub fn ctext() -> impl Parser<char, char, Error = Simple<char>> {
    filter(|c| matches!(u32::from(*c), 33..=39 | 42..=91 | 93..=126))
}

// comment         =       "(" *([FWS] ccontent) [FWS] ")"
pub fn comment() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    recursive(|comment| {
        // ccontent = ctext / quoted-pair / comment
        let ccontent = choice((
            ctext().repeated().exactly(1),
            quoted_pair().repeated().exactly(1),
            comment,
        ));

        fws()
            .or_not()
            .ignore_then(ccontent)
            .repeated()
            .flatten()
            .then_ignore(fws().or_not())
            .delimited_by(just('(').ignored(), just(')').ignored())
    })
}

// CFWS            =       *([FWS] comment) (([FWS] comment) / FWS)
pub fn cfws() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    fws()
        .or_not()
        .map(Option::unwrap_or_default)
        .chain(comment())
        .repeated()
        .flatten()
        .chain(choice((
            fws()
                .or_not()
                .map(Option::unwrap_or_default)
                .chain(comment()),
            fws(),
        )))
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
pub(super) fn atext() -> impl Parser<char, char, Error = Simple<char>> {
    filter(|c| {
        matches!(*c,
             'A'..='Z' |
             'a'..='z' |
             '0'..='9' |
             '!' | '#' |
             '$' | '%' |
             '&' | '\''|
             '*' | '+' |
             '-' | '/' |
             '=' | '?' |
             '^' | '_' |
             '`' | '{' |
             '|' | '}' |
             '~'
        )
    })
}

// atom            =       [CFWS] 1*atext [CFWS]
pub(super) fn atom() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    // NOTE: the last CFWS can be skipped since atoms are only used in
    // serie inside display names, so the last CFWS will always be
    // captured together with the first CFWS. This prevents to capture
    // trailing CFWS.
    cfws()
        .or_not()
        .map(Option::unwrap_or_default)
        .chain(
            // also allow non ASCII UTF8 chars
            choice((atext(), rfc5336::utf8_non_ascii()))
                .repeated()
                .at_least(1),
        )
        .chain(cfws().or_not().map(Option::unwrap_or_default))
}

// 3.2.5. Quoted strings
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.5

// qtext           =       NO-WS-CTL /     ; Non white space controls
//
//                         %d33 /          ; The rest of the US-ASCII
//                         %d35-91 /       ;  characters not including "\"
//                         %d93-126        ;  or the quote character
fn qtext() -> impl Parser<char, char, Error = Simple<char>> {
    choice((
        no_ws_ctl(),
        filter(|c| matches!(u32::from(*c), 33 | 35..=91 | 93..=126)),
    ))
}

// qcontent        =       qtext / quoted-pair
pub(super) fn qcontent() -> impl Parser<char, char, Error = Simple<char>> {
    choice((qtext(), quoted_pair()))
}

// quoted-string   =       [CFWS]
//                         DQUOTE *([FWS] qcontent) [FWS] DQUOTE
//                         [CFWS]
fn quoted_string() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    cfws()
        .or_not()
        .map(Option::unwrap_or_default)
        .chain(
            rfc2234::dquote()
                .ignore_then(
                    fws()
                        .or_not()
                        .map(Option::unwrap_or_default)
                        // also allow non ASCII UTF8 chars
                        .chain(choice((qcontent(), rfc5336::utf8_non_ascii())))
                        .repeated()
                        .flatten(),
                )
                .chain(fws().or_not().map(Option::unwrap_or_default))
                .then_ignore(rfc2234::dquote()),
        )
        .then_ignore(cfws().or_not().map(Option::unwrap_or_default))
}

// 3.2.6. Miscellaneous tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.6

// word            =       atom / quoted-string
fn word() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((atom(), quoted_string()))
}

// phrase          =       1*word / obs-phrase
fn phrase() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    // NOTE: obs_phrase also start by a word(), so it needs to be
    // tested first.
    choice((obs_phrase(), word().repeated().at_least(1).flatten()))
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
    display_name()
        .collect::<String>()
        // trim trailing CFWS and FWS from atoms and quoted strings
        .map(|name| name.trim().to_owned())
        .or_not()
        .then(angle_addr())
}

// angle-addr      =       [CFWS] "<" addr-spec ">" [CFWS] / obs-angle-addr
fn angle_addr() -> impl Parser<char, (String, String), Error = Simple<char>> {
    cfws()
        .or_not()
        .ignore_then(addr_spec().delimited_by(just('<').ignored(), just('>').ignored()))
        .then_ignore(cfws().or_not())
}

// display-name    =       phrase
fn display_name() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    phrase()
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

// 4.1. Miscellaneous obsolete tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-4.1

// obs-phrase      =       word *(word / "." / CFWS)
fn obs_phrase() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    word().chain(
        choice((word(), just('.').repeated().exactly(1), cfws()))
            .repeated()
            .flatten(),
    )
}
