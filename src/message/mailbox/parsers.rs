use chumsky::prelude::*;

const CR: char = 0x0D as char;
const LF: char = 0x0A as char;
const HTAB: char = 0x09 as char;
const SP: char = 0x20 as char;
const DQUOTE: char = 0x22 as char;
const DOT: char = 0x2E as char;

// 3.2.1 Primitive Tokens
// https://datatracker.ietf.org/doc/html/rfc2822#section-3-2-1

pub fn no_ws_ctl() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![];
    range.extend((1 as char)..=(8 as char));
    range.push(11 as char);
    range.push(12 as char);
    range.extend((14 as char)..=(31 as char));
    range.push(127 as char);
    one_of(range)
}

pub fn text() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![];
    range.extend((1 as char)..=(9 as char));
    range.push(11 as char);
    range.push(12 as char);
    range.extend((14 as char)..=(127 as char));
    one_of(range)
}

// 3.2.2 Quoted characters
// https://datatracker.ietf.org/doc/html/rfc2822#section-3-2-3

pub fn quoted_pair() -> impl Parser<char, char, Error = Simple<char>> {
    just('\\').ignore_then(text())
}

// 3.2.3 Folding white space and comments
// https://datatracker.ietf.org/doc/html/rfc2822#section-3-2-3

pub fn crlf() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    just(CR).chain(just(LF))
}

pub fn wsp() -> impl Parser<char, char, Error = Simple<char>> {
    one_of([SP, HTAB])
}

pub fn fws() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    wsp()
        .repeated()
        .chain(crlf())
        .or_not()
        .flatten()
        .chain(wsp().repeated().at_least(1))
}

pub fn ctext() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![];
    range.extend((33 as char)..=(39 as char));
    range.extend((42 as char)..=(91 as char));
    range.extend((93 as char)..=(126 as char));
    one_of(range)
}

pub fn comment() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    let ctext = ctext().map(|ctext| vec![ctext]);
    let quoted_pair = quoted_pair().map(|quoted_pair| vec![quoted_pair]);

    recursive(|comment| {
        fws()
            .or_not()
            .ignore_then(choice((ctext, quoted_pair, comment)))
            .repeated()
            .flatten()
            .then_ignore(fws().or_not())
            .delimited_by(just('(').ignored(), just(')').ignored())
    })
}

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

// 3.2.4 Atom
// https://datatracker.ietf.org/doc/html/rfc2822#section-3-2-4

pub fn dot_atom_text() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    atext().repeated().at_least(1).chain(
        just('.')
            .chain(atext().repeated().at_least(1))
            .repeated()
            .at_least(1)
            .flatten(),
    )
}

pub fn dot_atom() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    cfws()
        .or_not()
        .ignore_then(dot_atom_text())
        .then_ignore(cfws().or_not())
}

pub fn atom() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    cfws()
        .or_not()
        .flatten()
        .chain(atext().repeated().at_least(1))
        .chain(cfws().or_not().flatten())
}

// 3.2.5 Quoted strings
// https://datatracker.ietf.org/doc/html/rfc2822#section-3-2-5

pub fn atext() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![];
    range.extend('0'..='9');
    range.extend('a'..='z');
    range.extend('A'..='Z');
    range.extend("!#$%&\'*+-/=?^_`{|}~".chars());
    one_of(range)
}

pub fn qtext() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![33 as char];
    range.extend((35 as char)..=(91 as char));
    range.extend((93 as char)..=(126 as char));
    one_of(range)
}

pub fn qcontent() -> impl Parser<char, char, Error = Simple<char>> {
    choice((qtext(), quoted_pair()))
}

pub fn quoted_string() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    cfws()
        .or_not()
        .ignore_then(fws().or_not().ignore_then(qcontent()).repeated())
        .then_ignore(fws().or_not())
        .delimited_by(just(DQUOTE).ignored(), just(DQUOTE).ignored())
        .collect()
}

pub fn word() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((atom(), quoted_string()))
}

pub fn obs_phrase() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    word().chain(choice((word(), just(DOT).repeated().exactly(1), cfws())))
}

pub fn phrase() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((word().repeated().at_least(1).flatten(), obs_phrase()))
}

// 3.4. Address Specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3-4

pub fn mailbox() -> impl Parser<char, (Option<String>, String), Error = Simple<char>> {
    choice((name_addr(), addr_spec().collect().map(|addr| (None, addr))))
}

pub fn name_addr() -> impl Parser<char, (Option<String>, String), Error = Simple<char>> {
    display_name().or_not().then(angle_addr().collect())
}

pub fn angle_addr() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    cfws()
        .or_not()
        .ignore_then(addr_spec().delimited_by(just('<'), just('>')))
        .then_ignore(cfws().or_not())
}

pub fn display_name() -> impl Parser<char, String, Error = Simple<char>> {
    phrase().collect()
}

pub fn mailbox_list() -> impl Parser<char, Vec<(Option<String>, String)>, Error = Simple<char>> {
    mailbox().chain(just(',').ignore_then(mailbox()).repeated())
}

// 3.4.1. Addr-spec specification
// https://datatracker.ietf.org/doc/html/rfc2822#section-3-4-1

pub fn addr_spec() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    local_part().chain(just('@')).chain(domain())
}

pub fn local_part() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((dot_atom(), quoted_string(), obs_local_part()))
}

pub fn domain() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((dot_atom(), domain_literal(), obs_domain()))
}

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

pub fn dcontent() -> impl Parser<char, char, Error = Simple<char>> {
    choice((dtext(), quoted_pair()))
}

pub fn dtext() -> impl Parser<char, char, Error = Simple<char>> {
    let mut range: Vec<char> = vec![];
    range.extend((33 as char)..=(90 as char));
    range.extend((94 as char)..=(126 as char));
    choice((no_ws_ctl(), one_of(range)))
}

// 4.4. Obsolete Addressing
// https://datatracker.ietf.org/doc/html/rfc2822#section-4-4

pub fn obs_local_part() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    word().chain(just(DOT).chain(word()).repeated().flatten())
}

pub fn obs_domain() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    atom().chain(just(DOT).chain(atom()).repeated().flatten())
}
