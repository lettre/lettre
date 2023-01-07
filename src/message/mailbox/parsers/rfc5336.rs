//! Partial parsers implementation of [RFC5336]: SMTP Extension for
//! Internationalized Email Addresses.
//!
//! [RFC5336]: https://datatracker.ietf.org/doc/html/rfc5336

use chumsky::prelude::*;

use super::{rfc2234, rfc2821, rfc2822, rfc3629};

// 3.3.  Extended Mailbox Address Syntax
// https://datatracker.ietf.org/doc/html/rfc5336#section-3.3

// uMailbox = uLocal-part "@" uDomain
//   ; Replace Mailbox in RFC 2821, Section 4.1.2
pub(super) fn u_mailbox() -> impl Parser<char, (String, String), Error = Simple<char>> {
    u_local_part()
        .collect()
        .then_ignore(just('@'))
        .then(u_domain().collect())
}

// uLocal-part = uDot-string / uQuoted-string
//   ; MAY be case-sensitive
//   ; Replace Local-part in RFC 2821, Section 4.1.2
pub(super) fn u_local_part() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((u_dot_string(), u_quoted_string()))
}

// uDot-string = uAtom *("." uAtom)
//   ; Replace Dot-string in RFC 2821, Section 4.1.2
pub(super) fn u_dot_string() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    u_atom().chain(just('.').chain(u_atom()).repeated().flatten())
}

// uAtom = 1*ucharacter
//       ; Replace Atom in RFC 2821, Section 4.1.2
pub(super) fn u_atom() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    ucharacter().repeated().at_least(1).flatten()
}

// ucharacter = atext / UTF8-non-ascii
pub(super) fn ucharacter() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((rfc2822::atext().repeated().exactly(1), utf8_non_ascii()))
}

// uQuoted-string = DQUOTE *uqcontent DQUOTE
//   ; Replace Quoted-string in RFC 2821, Section 4.1.2
pub(super) fn u_quoted_string() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    rfc2234::dquote()
        .ignore_then(uqcontent().repeated().flatten())
        .then_ignore(rfc2234::dquote())
}

// uqcontent = qcontent / UTF8-non-ascii
pub(super) fn uqcontent() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((rfc2822::qcontent().repeated().exactly(1), utf8_non_ascii()))
}

// uDomain = (sub-udomain 1*("." sub-udomain)) / address-literal
//   ; Replace Domain in RFC 2821, Section 4.1.2
pub(super) fn u_domain() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    // TODO: missing address literal
    sub_udomain().chain(
        just('.')
            .chain(sub_udomain())
            .repeated()
            .at_least(1)
            .flatten(),
    )
}

// sub-udomain = uLet-dig [uLdh-str]
//   ; Replace sub-domain in RFC 2821, Section 4.1.2
pub(super) fn sub_udomain() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    u_let_dig().chain(u_ldh_str().or_not().map(Option::unwrap_or_default))
}

// uLet-dig = Let-dig / UTF8-non-ascii
pub(super) fn u_let_dig() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((rfc2821::let_dig().repeated().exactly(1), utf8_non_ascii()))
}

// uLdh-str = *( ALPHA / DIGIT / "-" / UTF8-non-ascii) uLet-dig
//   ; Replace Ldh-str in RFC 2821, Section 4.1.3
pub(super) fn u_ldh_str() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((
        rfc2234::alpha().repeated().exactly(1),
        rfc2234::digit().repeated().exactly(1),
        just('-').repeated().exactly(1),
        utf8_non_ascii(),
    ))
    .repeated()
    .flatten()
    .chain(u_let_dig())
}

// UTF8-non-ascii = UTF8-2 / UTF8-3 / UTF8-4
// UTF8-2 =  <See Section 4 of RFC 3629>
// UTF8-3 =  <See Section 4 of RFC 3629>
// UTF8-4 =  <See Section 4 of RFC 3629>
pub(super) fn utf8_non_ascii() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    choice((rfc3629::utf8_2(), rfc3629::utf8_3(), rfc3629::utf8_4()))
}

#[cfg(test)]
mod test {
    use chumsky::prelude::*;

    #[test]
    fn utf8_non_ascii() {
        println!("😂: {:?}", "😂".bytes());
        assert_eq!(vec!['5'], super::utf8_non_ascii().parse("😂").unwrap())
    }

    #[test]
    fn u_mailbox() {
        assert_eq!(
            ("coucou".into(), "localhost".into()),
            super::u_mailbox().parse("coucou@localhost").unwrap()
        )
    }
}
