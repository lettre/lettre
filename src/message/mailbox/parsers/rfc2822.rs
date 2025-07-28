//! Partial parsers implementation of [RFC2822]: Internet Message
//! Format.
//!
//! [RFC2822]: https://datatracker.ietf.org/doc/html/rfc2822

use nom::{
    branch::alt,
    character::complete::{char, satisfy},
    combinator::{eof, map, opt},
    multi::{fold_many0, fold_many1, many0, many1, separated_list0},
    sequence::{delimited, pair, preceded, terminated},
    IResult, Parser,
};

use super::{rfc2234, rfc5336};

// 3.2.1. Primitive Tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.1

// NO-WS-CTL       =       %d1-8 /         ; US-ASCII control characters
//                         %d11 /          ;  that do not include the
//                         %d12 /          ;  carriage return, line feed,
//                         %d14-31 /       ;  and white space characters
//                         %d127
fn no_ws_ctl(input: &str) -> IResult<&str, char> {
    satisfy(|c| matches!(u32::from(c), 1..=8 | 11 | 12 | 14..=31 | 127)).parse(input)
}

// text            =       %d1-9 /         ; Characters excluding CR and LF
//                         %d11 /
//                         %d12 /
//                         %d14-127 /
//                         obs-text
fn text(input: &str) -> IResult<&str, char> {
    satisfy(|c| matches!(u32::from(c), 1..=9 | 11 | 12 | 14..=127)).parse(input)
}

// 3.2.2. Quoted characters
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.2

// quoted-pair     =       ("\" text) / obs-qp
fn quoted_pair(input: &str) -> IResult<&str, char> {
    preceded(char('\\'), text).parse(input)
}

// 3.2.3. Folding white space and comments
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.3

// FWS             =       ([*WSP CRLF] 1*WSP) /   ; Folding white space
//                         obs-FWS
pub(super) fn fws(input: &str) -> IResult<&str, Option<char>> {
    map(
        pair(opt(rfc2234::wsp), many0(rfc2234::wsp)),
        |(first, _rest)| first,
    )
    .parse(input)
}

// CFWS            =       *([FWS] comment) (([FWS] comment) / FWS)
pub(super) fn cfws(input: &str) -> IResult<&str, Option<char>> {
    // TODO: comment are not currently supported, so for now a cfws is
    // the same as a fws.
    fws(input)
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
pub(super) fn atext(input: &str) -> IResult<&str, char> {
    alt((
        rfc2234::alpha,
        rfc2234::digit,
        satisfy(|c| {
            matches!(
                c,
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
        rfc5336::utf8_non_ascii,
    ))
    .parse(input)
}

// atom            =       [CFWS] 1*atext [CFWS]
pub(super) fn atom(input: &str) -> IResult<&str, String> {
    map(
        pair(
            cfws,
            fold_many1(atext, String::new, |mut acc, c| {
                acc.push(c);
                acc
            }),
        ),
        |(cfws, mut chars)| {
            if let Some(cfws) = cfws {
                chars.insert(0, cfws);
            }
            chars
        },
    )
    .parse(input)
}

// dot-atom        =       [CFWS] dot-atom-text [CFWS]
pub(super) fn dot_atom(input: &str) -> IResult<&str, String> {
    map(pair(cfws, dot_atom_text), |(_cfws, text)| text).parse(input)
}

// dot-atom-text   =       1*atext *("." 1*atext)
pub(super) fn dot_atom_text(input: &str) -> IResult<&str, String> {
    map(
        pair(
            fold_many1(atext, String::new, |mut acc, c| {
                acc.push(c);
                acc
            }),
            many0(map(
                pair(
                    char('.'),
                    fold_many1(atext, String::new, |mut acc, c| {
                        acc.push(c);
                        acc
                    }),
                ),
                |(dot, chars)| format!("{dot}{chars}"),
            )),
        ),
        |(first, rest)| {
            let mut result = first;
            for part in rest {
                result.push_str(&part);
            }
            result
        },
    )
    .parse(input)
}

// 3.2.5. Quoted strings
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.5

// qtext           =       NO-WS-CTL /     ; Non white space controls
//
//                         %d33 /          ; The rest of the US-ASCII
//                         %d35-91 /       ;  characters not including "\"
//                         %d93-126        ;  or the quote character
fn qtext(input: &str) -> IResult<&str, char> {
    alt((
        satisfy(|c| matches!(u32::from(c), 33 | 35..=91 | 93..=126)),
        no_ws_ctl,
    ))
    .parse(input)
}

// qcontent        =       qtext / quoted-pair
pub(super) fn qcontent(input: &str) -> IResult<&str, char> {
    alt((qtext, quoted_pair, rfc5336::utf8_non_ascii)).parse(input)
}

// quoted-string   =       [CFWS]
//                         DQUOTE *([FWS] qcontent) [FWS] DQUOTE
//                         [CFWS]
fn quoted_string(input: &str) -> IResult<&str, String> {
    map(
        delimited(
            rfc2234::dquote,
            fold_many0(
                map(pair(fws, qcontent), |(_fws, c)| c),
                String::new,
                |mut acc, c| {
                    acc.push(c);
                    acc
                },
            ),
            preceded(many0(satisfy(char::is_whitespace)), rfc2234::dquote),
        ),
        |s| s,
    )
    .parse(input)
}

// 3.2.6. Miscellaneous tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.6

// word            =       atom / quoted-string
fn word(input: &str) -> IResult<&str, String> {
    alt((quoted_string, atom)).parse(input)
}

// phrase          =       1*word / obs-phrase
fn phrase(input: &str) -> IResult<&str, String> {
    alt((obs_phrase, map(many1(word), |words| words.join(" ")))).parse(input)
}

// 3.4. Address Specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4

// mailbox         =       name-addr / addr-spec
pub(crate) fn mailbox(input: &str) -> IResult<&str, (Option<String>, (String, String))> {
    terminated(alt((name_addr, map(addr_spec, |addr| (None, addr)))), eof).parse(input)
}

// name-addr       =       [display-name] angle-addr
fn name_addr(input: &str) -> IResult<&str, (Option<String>, (String, String))> {
    pair(opt(display_name), angle_addr).parse(input)
}

// angle-addr      =       [CFWS] "<" addr-spec ">" [CFWS] / obs-angle-addr
fn angle_addr(input: &str) -> IResult<&str, (String, String)> {
    delimited((cfws, char('<')), addr_spec, (char('>'), cfws)).parse(input)
}

// display-name    =       phrase
fn display_name(input: &str) -> IResult<&str, String> {
    phrase(input)
}

// mailbox-list    =       (mailbox *("," mailbox)) / obs-mbox-list
#[allow(clippy::type_complexity)]
pub(crate) fn mailbox_list(input: &str) -> IResult<&str, Vec<(Option<String>, (String, String))>> {
    terminated(
        separated_list0(
            delimited(
                many0(satisfy(char::is_whitespace)),
                char(','),
                many0(satisfy(char::is_whitespace)),
            ),
            alt((name_addr, map(addr_spec, |addr| (None, addr)))),
        ),
        eof,
    )
    .parse(input)
}

// 3.4.1. Addr-spec specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4.1

// addr-spec       =       local-part "@" domain
pub(super) fn addr_spec(input: &str) -> IResult<&str, (String, String)> {
    pair(terminated(local_part, char('@')), domain).parse(input)
}

// local-part      =       dot-atom / quoted-string / obs-local-part
pub(super) fn local_part(input: &str) -> IResult<&str, String> {
    alt((dot_atom, quoted_string, obs_local_part)).parse(input)
}

// domain          =       dot-atom / domain-literal / obs-domain
pub(super) fn domain(input: &str) -> IResult<&str, String> {
    // NOTE: omitting domain-literal since it may never be used
    alt((dot_atom, obs_domain)).parse(input)
}

// 4.1. Miscellaneous obsolete tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-4.1

// obs-phrase      =       word *(word / "." / CFWS)
fn obs_phrase(input: &str) -> IResult<&str, String> {
    // NOTE: the CFWS is already captured by the word, no need to add
    // it there.
    map(
        pair(word, many0(alt((word, map(char('.'), |c| c.to_string()))))),
        |(first, rest)| {
            let mut result = first;
            for part in rest {
                result.push_str(&part);
            }
            result
        },
    )
    .parse(input)
}

// 4.4. Obsolete Addressing
// https://datatracker.ietf.org/doc/html/rfc2822#section-4.4

// obs-local-part  =       word *("." word)
pub(super) fn obs_local_part(input: &str) -> IResult<&str, String> {
    map(
        pair(
            word,
            many0(map(pair(char('.'), word), |(dot, w)| format!("{dot}{w}"))),
        ),
        |(first, rest)| {
            let mut result = first;
            for part in rest {
                result.push_str(&part);
            }
            result
        },
    )
    .parse(input)
}

// obs-domain      =       atom *("." atom)
pub(super) fn obs_domain(input: &str) -> IResult<&str, String> {
    map(
        pair(
            atom,
            many0(map(pair(char('.'), atom), |(dot, a)| format!("{dot}{a}"))),
        ),
        |(first, rest)| {
            let mut result = first;
            for part in rest {
                result.push_str(&part);
            }
            result
        },
    )
    .parse(input)
}
