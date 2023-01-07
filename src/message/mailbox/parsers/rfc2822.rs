//! Partial parsers implementation of [RFC2822]: Internet Message
//! Format.
//!
//! [RFC2822]: https://datatracker.ietf.org/doc/html/rfc2822

use chumsky::prelude::*;

use super::{rfc2234, rfc5336};

// Primitive Tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.1

// NO-WS-CTL       =       %d1-8 /         ; US-ASCII control characters
//                         %d11 /          ;  that do not include the
//                         %d12 /          ;  carriage return, line feed,
//                         %d14-31 /       ;  and white space characters
//                         %d127
pub fn no_ws_ctl() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![];
    range.extend((1 as char)..=(8 as char));
    range.push(11 as char);
    range.push(12 as char);
    range.extend((14 as char)..=(31 as char));
    range.push(127 as char);
    one_of(range)
}

// text            =       %d1-9 /         ; Characters excluding CR and LF
//                         %d11 /
//                         %d12 /
//                         %d14-127 /
//                         obs-text
pub fn text() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![];
    range.extend((1 as char)..=(9 as char));
    range.push(11 as char);
    range.push(12 as char);
    range.extend((14 as char)..=(127 as char));
    one_of(range)
}

// Quoted characters
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.2

// quoted-pair     =       ("\" text) / obs-qp
pub fn quoted_pair() -> impl Parser<char, char, Error = Simple<char>> {
    choice((just('\\').ignore_then(text()), obs_qp()))
}

// Folding white space and comments
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
    let mut range: Vec<char> = vec![];
    range.extend((33 as char)..=(39 as char));
    range.extend((42 as char)..=(91 as char));
    range.extend((93 as char)..=(126 as char));
    one_of(range)
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
    fws().or(fws()
        .or_not()
        .flatten()
        .chain(comment())
        .repeated()
        .at_least(1)
        .flatten()
        .chain(fws().or_not().flatten()))
}

// Atom
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
pub fn atext() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![];
    range.extend('0'..='9');
    range.extend('a'..='z');
    range.extend('A'..='Z');
    range.extend("!#$%&\'*+-/=?^_`{|}~".chars());
    one_of(range)
}

// atom            =       [CFWS] 1*atext [CFWS]
pub fn atom() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    cfws()
        .or_not()
        .ignore_then(atext().repeated().at_least(1))
        .then_ignore(cfws().or_not())
}

// dot-atom        =       [CFWS] dot-atom-text [CFWS]
pub fn dot_atom() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    cfws()
        .or_not()
        .ignore_then(dot_atom_text())
        .then_ignore(cfws().or_not())
}

// dot-atom-text   =       1*atext *("." 1*atext)
pub fn dot_atom_text() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    atext().repeated().at_least(1).chain(
        just('.')
            .chain(atext().repeated().at_least(1))
            .repeated()
            .at_least(1)
            .flatten(),
    )
}

// Quoted strings
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.5

// qtext           =       NO-WS-CTL /     ; Non white space controls
//
//                         %d33 /          ; The rest of the US-ASCII
//                         %d35-91 /       ;  characters not including "\"
//                         %d93-126        ;  or the quote character
pub fn qtext() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![33 as char];
    range.extend((35 as char)..=(91 as char));
    range.extend((93 as char)..=(126 as char));
    one_of(range)
}

// qcontent        =       qtext / quoted-pair
pub fn qcontent() -> impl Parser<char, char, Error = Simple<char>> {
    choice((qtext(), quoted_pair()))
}

// quoted-string   =       [CFWS]
//                         DQUOTE *([FWS] qcontent) [FWS] DQUOTE
//                         [CFWS]
pub fn quoted_string() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    cfws()
        .or_not()
        .ignore_then(fws().or_not().ignore_then(qcontent()).repeated())
        .then_ignore(fws().or_not())
        .delimited_by(just('"').ignored(), just('"').ignored())
        .collect()
}

// 3.2.6. Miscellaneous tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.6

// word            =       atom / quoted-string
pub fn word() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((atom(), quoted_string()))
}

// Address Specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4

// mailbox-list    =       (mailbox *("," mailbox)) / obs-mbox-list
pub fn mailbox_list(
) -> impl Parser<char, Vec<(Option<String>, (String, String))>, Error = Simple<char>> {
    mailbox().chain(just(',').ignore_then(mailbox()).repeated())
}

// Addr-spec specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4.1

// Miscellaneous obsolete tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-4.1

// obs-qp          =       "\" (%d0-127)
pub fn obs_qp() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![];
    range.extend((0 as char)..=(127 as char));
    just('\\').ignore_then(one_of(range))
}

// Obsolete Addressing
// https://datatracker.ietf.org/doc/html/rfc2822#section-4.4

// obs-local-part  =       word *("." word)
pub fn obs_local_part() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    word().chain(just('.').chain(word()).repeated().flatten())
}

// obs-domain      =       atom *("." atom)
pub fn obs_domain() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    atom().chain(just('.').chain(atom()).repeated().flatten())
}

// -----------------------------------------------------

// Address Specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4

// mailbox         =       name-addr / addr-spec
pub(crate) fn mailbox(
) -> impl Parser<char, (Option<String>, (String, String)), Error = Simple<char>> {
    choice((addr_spec().map(|addr| (None, addr)), name_addr()))
}

// addr-spec       =       local-part "@" domain
pub fn addr_spec() -> impl Parser<char, (String, String), Error = Simple<char>> {
    // NOTE: we use the rfc5336 unicode mailbox spec instead of the
    // rfc2822 address spec because in `lettre` context headers are
    // already pre-parsed and decoded.
    rfc5336::u_mailbox()
}

// name-addr       =       [display-name] angle-addr
pub fn name_addr() -> impl Parser<char, (Option<String>, (String, String)), Error = Simple<char>> {
    display_name().or_not().then(angle_addr())
    // .map(|(display_name, address)| {
    //     (
    //         if display_name.is_empty() {
    //             None
    //         } else {
    //             Some(String::from_iter(display_name))
    //         },
    //         address,
    //     )
    // })
}

// angle-addr      =       [CFWS] "<" addr-spec ">" [CFWS] / obs-angle-addr
pub fn angle_addr() -> impl Parser<char, (String, String), Error = Simple<char>> {
    cfws()
        .or_not()
        .ignore_then(addr_spec().delimited_by(just('<').ignored(), just('>').ignored()))
        .then_ignore(cfws().or_not())
}

// display-name    =       phrase
// phrase          =       1*word / obs-phrase
// word            =       atom / quoted-string
pub fn display_name() -> impl Parser<char, String, Error = Simple<char>> {
    // NOTE: we use the rfc5336 unicode atom and quoted string spec
    // instead of the rfc2822 ones because in `lettre` context headers
    // are already pre-parsed and decoded.
    choice((rfc5336::u_atom(), rfc5336::u_quoted_string()))
        .repeated()
        .at_least(1)
        .flatten()
        .collect()
}
