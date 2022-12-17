use chumsky::prelude::*;

// Core Rules
// https://datatracker.ietf.org/doc/html/rfc2234#section-6.1

// CRLF           =  CR LF
//                        ; Internet standard newline
pub fn crlf() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    just('\r').chain(just('\n'))
}

// WSP            =  SP / HTAB
//                        ; white space
pub fn wsp() -> impl Parser<char, char, Error = Simple<char>> {
    one_of([' ', '\t'])
}

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
    wsp()
        .repeated()
        .chain(crlf())
        .or_not()
        .flatten()
        .chain(wsp().repeated().at_least(1))
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

// Miscellaneous tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.6

// word            =       atom / quoted-string
pub fn word() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((atom(), quoted_string()))
}

// Address Specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4

// mailbox         =       name-addr / addr-spec
pub fn mailbox() -> impl Parser<char, (Option<String>, (String, String)), Error = Simple<char>> {
    choice((addr_spec().map(|addr| (None, addr)), name_addr()))
}

// name-addr       =       [display-name] angle-addr
pub fn name_addr() -> impl Parser<char, (Option<String>, (String, String)), Error = Simple<char>> {
    // NOTE: display-name does not follow the RFC here in order to be
    // more flexible.
    cfws().or_not().ignore_then(just('"').or_not()).ignore_then(
        take_until(just('"').or_not().ignore_then(angle_addr())).map(|(display_name, address)| {
            (
                if display_name.is_empty() {
                    None
                } else {
                    Some(String::from_iter(display_name))
                },
                address,
            )
        }),
    )
}

// angle-addr      =       [CFWS] "<" addr-spec ">" [CFWS] / obs-angle-addr
pub fn angle_addr() -> impl Parser<char, (String, String), Error = Simple<char>> {
    cfws()
        .or_not()
        .ignore_then(addr_spec().delimited_by(just('<').ignored(), just('>').ignored()))
        .then_ignore(cfws().or_not())
}

// mailbox-list    =       (mailbox *("," mailbox)) / obs-mbox-list
pub fn mailbox_list(
) -> impl Parser<char, Vec<(Option<String>, (String, String))>, Error = Simple<char>> {
    mailbox().chain(just(',').ignore_then(mailbox()).repeated())
}

// Addr-spec specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3.4.1

// addr-spec       =       local-part "@" domain
pub fn addr_spec() -> impl Parser<char, (String, String), Error = Simple<char>> {
    local_part()
        .collect()
        .then_ignore(just('@'))
        .then(domain().collect())
}

// local-part      =       dot-atom / quoted-string / obs-local-part
pub fn local_part() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((dot_atom(), quoted_string(), obs_local_part()))
}

// domain          =       dot-atom / domain-literal / obs-domain
pub fn domain() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((dot_atom(), domain_literal(), obs_domain()))
}

// domain-literal  =       [CFWS] "[" *([FWS] dcontent) [FWS] "]" [CFWS]
pub fn domain_literal() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    cfws()
        .or_not()
        .ignore_then(
            fws()
                .or_not()
                .ignore_then(dcontent())
                .repeated()
                .then_ignore(fws().or_not())
                .delimited_by(just('[').ignored(), just(']').ignored()),
        )
        .then_ignore(cfws().or_not())
}

// dcontent        =       dtext / quoted-pair
pub fn dcontent() -> impl Parser<char, char, Error = Simple<char>> {
    choice((dtext(), quoted_pair()))
}

// dtext           =       NO-WS-CTL /     ; Non white space controls
//
//                         %d33-90 /       ; The rest of the US-ASCII
//                         %d94-126        ;  characters not including "[",
//                                         ;  "]", or "\"
pub fn dtext() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![];
    range.extend((33 as char)..=(90 as char));
    range.extend((94 as char)..=(126 as char));
    choice((no_ws_ctl(), one_of(range)))
}

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
