//! SMTP response, containing a mandatory return code and an optional text
//! message

use std::{
    fmt::{Display, Formatter, Result},
    result,
    str::FromStr,
};

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::streaming::{tag, take_until},
    combinator::{complete, map},
    multi::many0,
    sequence::preceded,
};

use crate::transport::smtp::{Error, error};

/// The first digit indicates severity
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// 2yx
    PositiveCompletion = 2,
    /// 3yz
    PositiveIntermediate = 3,
    /// 4yz
    TransientNegativeCompletion = 4,
    /// 5yz
    PermanentNegativeCompletion = 5,
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", *self as u8)
    }
}

/// Second digit
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Category {
    /// x0z
    Syntax = 0,
    /// x1z
    Information = 1,
    /// x2z
    Connections = 2,
    /// x3z
    Unspecified3 = 3,
    /// x4z
    Unspecified4 = 4,
    /// x5z
    MailSystem = 5,
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", *self as u8)
    }
}

/// The detail digit of a response code (third digit)
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Detail {
    #[allow(missing_docs)]
    Zero = 0,
    #[allow(missing_docs)]
    One = 1,
    #[allow(missing_docs)]
    Two = 2,
    #[allow(missing_docs)]
    Three = 3,
    #[allow(missing_docs)]
    Four = 4,
    #[allow(missing_docs)]
    Five = 5,
    #[allow(missing_docs)]
    Six = 6,
    #[allow(missing_docs)]
    Seven = 7,
    #[allow(missing_docs)]
    Eight = 8,
    #[allow(missing_docs)]
    Nine = 9,
}

impl Display for Detail {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", *self as u8)
    }
}

/// Represents a 3 digit SMTP response code
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Code {
    /// First digit of the response code
    pub severity: Severity,
    /// Second digit of the response code
    pub category: Category,
    /// Third digit
    pub detail: Detail,
}

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}{}{}", self.severity, self.category, self.detail)
    }
}

impl Code {
    /// Creates a new `Code` structure
    pub fn new(severity: Severity, category: Category, detail: Detail) -> Code {
        Code {
            severity,
            category,
            detail,
        }
    }

    /// Tells if the response is positive
    pub fn is_positive(self) -> bool {
        matches!(
            self.severity,
            Severity::PositiveCompletion | Severity::PositiveIntermediate
        )
    }
}

impl From<Code> for u16 {
    fn from(code: Code) -> Self {
        code.detail as u16 + 10 * code.category as u16 + 100 * code.severity as u16
    }
}

/// Contains an SMTP reply, with separated code and message
///
/// The text message is optional, only the code is mandatory
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Response {
    /// Response code
    code: Code,
    /// Server response string (optional)
    /// Handle multiline responses
    message: Vec<String>,
}

impl FromStr for Response {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Response, Error> {
        parse_response(s)
            .map(|(_, r)| r)
            .map_err(|e| error::response(e.to_owned()))
    }
}

impl Response {
    /// Creates a new `Response`
    pub fn new(code: Code, message: Vec<String>) -> Response {
        Response { code, message }
    }

    /// Tells if the response is positive
    pub fn is_positive(&self) -> bool {
        self.code.is_positive()
    }

    /// Tests code equality
    pub fn has_code(&self, code: u16) -> bool {
        self.code.to_string() == code.to_string()
    }

    /// Returns only the first word of the message if possible
    pub fn first_word(&self) -> Option<&str> {
        self.message
            .first()
            .and_then(|line| line.split_whitespace().next())
    }

    /// Returns only the line of the message if possible
    pub fn first_line(&self) -> Option<&str> {
        self.message.first().map(String::as_str)
    }

    /// Response code
    pub fn code(&self) -> Code {
        self.code
    }

    /// Server response string (array of lines)
    pub fn message(&self) -> impl Iterator<Item = &str> {
        self.message.iter().map(String::as_str)
    }

    /// The message lines joined with spaces, keeping only the first
    /// occurrence of a repeated enhanced status code.
    pub(super) fn message_joined(&self) -> String {
        let Some((first, rest)) = self.message.split_first() else {
            return String::new();
        };

        let repeated = self.enhanced_status_code();
        let mut joined = String::from(first);
        for line in rest {
            joined.push(' ');
            joined.push_str(match repeated {
                Some(code) => line
                    .strip_prefix(code)
                    .filter(|rest| rest.starts_with(' '))
                    .map_or(line.as_str(), str::trim_start),
                None => line.as_str(),
            });
        }
        joined
    }

    /// The enhanced status code the reply starts with, if it has one.
    ///
    /// RFC 2034 section 4: the code must agree with the reply code.
    fn enhanced_status_code(&self) -> Option<&str> {
        let expected_class = match self.code.severity {
            Severity::PositiveCompletion => "2",
            Severity::TransientNegativeCompletion => "4",
            Severity::PermanentNegativeCompletion => "5",
            // RFC 2034 defines classes 2, 4 and 5 only.
            Severity::PositiveIntermediate => return None,
        };

        let word = self.first_word()?;
        let (class, rest) = word.split_once('.')?;
        let (subject, detail) = rest.split_once('.')?;

        (class == expected_class
            && [subject, detail].into_iter().all(|part| {
                (1..=3).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_digit())
            }))
        .then_some(word)
    }
}

// Parsers (originally from tokio-smtp)

fn parse_code(i: &str) -> IResult<&str, Code> {
    let (i, severity) = parse_severity(i)?;
    let (i, category) = parse_category(i)?;
    let (i, detail) = parse_detail(i)?;
    Ok((
        i,
        Code {
            severity,
            category,
            detail,
        },
    ))
}

fn parse_severity(i: &str) -> IResult<&str, Severity> {
    alt((
        map(tag("2"), |_| Severity::PositiveCompletion),
        map(tag("3"), |_| Severity::PositiveIntermediate),
        map(tag("4"), |_| Severity::TransientNegativeCompletion),
        map(tag("5"), |_| Severity::PermanentNegativeCompletion),
    ))
    .parse(i)
}

fn parse_category(i: &str) -> IResult<&str, Category> {
    alt((
        map(tag("0"), |_| Category::Syntax),
        map(tag("1"), |_| Category::Information),
        map(tag("2"), |_| Category::Connections),
        map(tag("3"), |_| Category::Unspecified3),
        map(tag("4"), |_| Category::Unspecified4),
        map(tag("5"), |_| Category::MailSystem),
    ))
    .parse(i)
}

fn parse_detail(i: &str) -> IResult<&str, Detail> {
    alt((
        map(tag("0"), |_| Detail::Zero),
        map(tag("1"), |_| Detail::One),
        map(tag("2"), |_| Detail::Two),
        map(tag("3"), |_| Detail::Three),
        map(tag("4"), |_| Detail::Four),
        map(tag("5"), |_| Detail::Five),
        map(tag("6"), |_| Detail::Six),
        map(tag("7"), |_| Detail::Seven),
        map(tag("8"), |_| Detail::Eight),
        map(tag("9"), |_| Detail::Nine),
    ))
    .parse(i)
}

pub(crate) fn parse_response(i: &str) -> IResult<&str, Response> {
    let (i, lines) = many0((
        parse_code,
        preceded(tag("-"), take_until("\r\n")),
        tag("\r\n"),
    ))
    .parse(i)?;
    let (i, (last_code, last_line)) =
        (parse_code, preceded(tag(" "), take_until("\r\n"))).parse(i)?;
    let (i, _) = complete(tag("\r\n")).parse(i)?;

    // Check that all codes are equal.
    if !lines.iter().all(|&(code, _, _)| code == last_code) {
        return Err(nom::Err::Failure(nom::error::Error::new(
            "",
            nom::error::ErrorKind::Not,
        )));
    }

    // Extract text from lines, and append last line.
    let mut lines: Vec<String> = lines.into_iter().map(|(_, text, _)| text.into()).collect();
    lines.push(last_line.into());

    Ok((
        i,
        Response {
            code: last_code,
            message: lines,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_multiline_error_keeps_lines_separated() {
        // Gmail's reverse-DNS rejection: every line repeats the status code.
        let raw = "550-5.7.25 [203.0.113.1] The IP address sending this message does not have a\r\n\
                   550-5.7.25 PTR record setup, or the corresponding forward DNS entry does not\r\n\
                   550 5.7.25 match the sending IP.\r\n";

        let response: Response = raw.parse().unwrap();
        let rendered = error::code_from_response(&response).to_string();

        assert!(
            rendered.ends_with(
                "5.7.25 [203.0.113.1] The IP address sending this message does not have a \
                 PTR record setup, or the corresponding forward DNS entry does not \
                 match the sending IP."
            ),
            "unexpected rendering: {rendered}"
        );
        assert_eq!(
            rendered.matches("5.7.25").count(),
            1,
            "status code repeated: {rendered}"
        );
    }

    #[test]
    fn test_multiline_message_without_enhanced_code() {
        let raw = "250-first line\r\n\
                   250 second line\r\n";

        let response: Response = raw.parse().unwrap();
        assert_eq!(response.message_joined(), "first line second line");
    }

    #[test]
    fn test_status_code_prefixing_a_longer_one_is_kept() {
        // Stripping "5.7.2" off the second line would leave a stray "5".
        let raw = "550-5.7.2 first line\r\n\
                   550 5.7.25 second line\r\n";

        let response: Response = raw.parse().unwrap();
        assert_eq!(
            response.message_joined(),
            "5.7.2 first line 5.7.25 second line"
        );
    }

    #[test]
    fn test_enhanced_code_disagreeing_with_reply_code_is_kept() {
        // Non-conformant: the classes disagree, so nothing is stripped.
        let raw = "550-4.7.1 first line\r\n\
                   550 4.7.1 second line\r\n";

        let response: Response = raw.parse().unwrap();
        assert_eq!(
            response.message_joined(),
            "4.7.1 first line 4.7.1 second line"
        );
    }

    #[test]
    fn test_severity_fmt() {
        assert_eq!(format!("{}", Severity::PositiveCompletion), "2");
    }

    #[test]
    fn test_category_fmt() {
        assert_eq!(format!("{}", Category::Unspecified4), "4");
    }

    #[test]
    fn test_code_new() {
        assert_eq!(
            Code::new(
                Severity::TransientNegativeCompletion,
                Category::Connections,
                Detail::Zero,
            ),
            Code {
                severity: Severity::TransientNegativeCompletion,
                category: Category::Connections,
                detail: Detail::Zero,
            }
        );
    }

    #[test]
    fn test_code_display() {
        let code = Code {
            severity: Severity::TransientNegativeCompletion,
            category: Category::Connections,
            detail: Detail::One,
        };

        assert_eq!(code.to_string(), "421");
    }

    #[test]
    fn test_code_to_u16() {
        let code = Code {
            severity: Severity::TransientNegativeCompletion,
            category: Category::Connections,
            detail: Detail::One,
        };
        let c: u16 = code.into();
        assert_eq!(c, 421);
    }

    #[test]
    fn test_response_from_str() {
        let raw_response = "250-me\r\n250-8BITMIME\r\n250-SIZE 42\r\n250 AUTH PLAIN CRAM-MD5\r\n";
        assert_eq!(
            raw_response.parse::<Response>().unwrap(),
            Response {
                code: Code {
                    severity: Severity::PositiveCompletion,
                    category: Category::MailSystem,
                    detail: Detail::Zero,
                },
                message: vec![
                    "me".to_owned(),
                    "8BITMIME".to_owned(),
                    "SIZE 42".to_owned(),
                    "AUTH PLAIN CRAM-MD5".to_owned(),
                ],
            }
        );

        let wrong_code = "2506-me\r\n250-8BITMIME\r\n250-SIZE 42\r\n250 AUTH PLAIN CRAM-MD5\r\n";
        assert!(wrong_code.parse::<Response>().is_err());

        let wrong_end = "250-me\r\n250-8BITMIME\r\n250-SIZE 42\r\n250-AUTH PLAIN CRAM-MD5\r\n";
        assert!(wrong_end.parse::<Response>().is_err());
    }

    #[test]
    fn test_response_is_positive() {
        assert!(
            Response::new(
                Code {
                    severity: Severity::PositiveCompletion,
                    category: Category::MailSystem,
                    detail: Detail::Zero,
                },
                vec!["me".to_owned(), "8BITMIME".to_owned(), "SIZE 42".to_owned(),],
            )
            .is_positive()
        );
        assert!(
            !Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::Zero,
                },
                vec!["me".to_owned(), "8BITMIME".to_owned(), "SIZE 42".to_owned(),],
            )
            .is_positive()
        );
    }

    #[test]
    fn test_response_has_code() {
        assert!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec!["me".to_owned(), "8BITMIME".to_owned(), "SIZE 42".to_owned(),],
            )
            .has_code(451)
        );
        assert!(
            !Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec!["me".to_owned(), "8BITMIME".to_owned(), "SIZE 42".to_owned(),],
            )
            .has_code(251)
        );
    }

    #[test]
    fn test_response_first_word() {
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec!["me".to_owned(), "8BITMIME".to_owned(), "SIZE 42".to_owned(),],
            )
            .first_word(),
            Some("me")
        );
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec![
                    "me mo".to_owned(),
                    "8BITMIME".to_owned(),
                    "SIZE 42".to_owned(),
                ],
            )
            .first_word(),
            Some("me")
        );
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec![],
            )
            .first_word(),
            None
        );
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec![" ".to_owned()],
            )
            .first_word(),
            None
        );
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec!["  ".to_owned()],
            )
            .first_word(),
            None
        );
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec!["".to_owned()],
            )
            .first_word(),
            None
        );
    }

    #[test]
    fn test_response_incomplete() {
        let raw_response = "250-smtp.example.org\r\n";
        let res = parse_response(raw_response);
        match res {
            Err(nom::Err::Incomplete(_)) => {}
            _ => panic!("Expected incomplete response, got {res:?}"),
        }
    }

    #[test]
    fn test_response_first_line() {
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec!["me".to_owned(), "8BITMIME".to_owned(), "SIZE 42".to_owned(),],
            )
            .first_line(),
            Some("me")
        );
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec![
                    "me mo".to_owned(),
                    "8BITMIME".to_owned(),
                    "SIZE 42".to_owned(),
                ],
            )
            .first_line(),
            Some("me mo")
        );
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec![],
            )
            .first_line(),
            None
        );
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec![" ".to_owned()],
            )
            .first_line(),
            Some(" ")
        );
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec!["  ".to_owned()],
            )
            .first_line(),
            Some("  ")
        );
        assert_eq!(
            Response::new(
                Code {
                    severity: Severity::TransientNegativeCompletion,
                    category: Category::MailSystem,
                    detail: Detail::One,
                },
                vec!["".to_owned()],
            )
            .first_line(),
            Some("")
        );
    }
}
