use mime::Mime;

use super::{Header, HeaderName};
use crate::BoxError;
use std::{
    convert::TryFrom,
    error::Error as StdError,
    fmt::{self, Display, Formatter as FmtFormatter, Result as FmtResult},
    str::FromStr,
};

#[derive(Clone)]
pub struct ContentType(Mime);

impl ContentType {
    pub fn parse(s: &str) -> Result<Self, ContentTypeErr> {
        s.parse().map(Self).map_err(ContentTypeErr)
    }

    pub(crate) fn from_mime(mime: Mime) -> Self {
        Self(mime)
    }

    pub(crate) fn as_ref(&self) -> &Mime {
        &self.0
    }
}

impl Header for ContentType {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_static("Content-Type")
    }

    fn parse_value(s: &str) -> Result<Self, BoxError> {
        Ok(Self(s.parse()?))
    }

    fn display(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug)]
pub struct ContentTypeErr(mime::FromStrError);

impl FromStr for ContentType {
    type Err = ContentTypeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl<'a> TryFrom<&'a str> for ContentType {
    type Error = ContentTypeErr;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl StdError for ContentTypeErr {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.0)
    }
}

impl Display for ContentTypeErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// `Content-Transfer-Encoding` of the body
///
/// The `Message` builder takes care of choosing the most
/// efficient encoding based on the chosen body, so in most
/// use-caches this header shouldn't be set manually.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ContentTransferEncoding {
    SevenBit,
    QuotedPrintable,
    Base64,
    // 8BITMIME
    EightBit,
    Binary,
}

impl Default for ContentTransferEncoding {
    fn default() -> Self {
        ContentTransferEncoding::Base64
    }
}

impl Display for ContentTransferEncoding {
    fn fmt(&self, f: &mut FmtFormatter<'_>) -> FmtResult {
        f.write_str(match *self {
            Self::SevenBit => "7bit",
            Self::QuotedPrintable => "quoted-printable",
            Self::Base64 => "base64",
            Self::EightBit => "8bit",
            Self::Binary => "binary",
        })
    }
}

impl FromStr for ContentTransferEncoding {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "7bit" => Ok(Self::SevenBit),
            "quoted-printable" => Ok(Self::QuotedPrintable),
            "base64" => Ok(Self::Base64),
            "8bit" => Ok(Self::EightBit),
            "binary" => Ok(Self::Binary),
            _ => Err(s.into()),
        }
    }
}

impl Header for ContentTransferEncoding {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_static("Content-Transfer-Encoding")
    }

    fn parse_value(s: &str) -> Result<Self, BoxError> {
        Ok(s.parse()?)
    }

    fn display(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct ContentDisposition {
    pub disposition: DispositionType,
    pub file_name: Option<String>,
}

#[derive(Debug, Copy, Clone)]
pub enum DispositionType {
    Inline,
    Attachment,
}

impl Header for ContentDisposition {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_static("Content-Disposition")
    }

    fn parse_value(s: &str) -> Result<Self, BoxError> {
        todo!()
    }

    fn display(&self) -> String {
        let type_str = match &self.disposition {
            DispositionType::Inline => "inline",
            DispositionType::Attachment => "attachment",
        };

        match &self.file_name {
            Some(file_name) => {
                format!("{}; filename=\"{}\"", type_str, file_name)
            }
            None => type_str.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::ContentTransferEncoding;
    use crate::message::header::{HeaderName, Headers};

    #[test]
    fn format_content_transfer_encoding() {
        let mut headers = Headers::new();

        headers.set(ContentTransferEncoding::SevenBit);

        assert_eq!(
            format!("{}", headers),
            "Content-Transfer-Encoding: 7bit\r\n"
        );

        headers.set(ContentTransferEncoding::Base64);

        assert_eq!(
            format!("{}", headers),
            "Content-Transfer-Encoding: base64\r\n"
        );
    }

    #[test]
    fn parse_content_transfer_encoding() {
        let mut headers = Headers::new();

        headers.set_raw(
            HeaderName::new_from_ascii_static("Content-Transfer-Encoding"),
            "7bit".into(),
        );

        assert_eq!(
            headers.get::<ContentTransferEncoding>(),
            Some(ContentTransferEncoding::SevenBit)
        );

        headers.set_raw(
            HeaderName::new_from_ascii_static("Content-Transfer-Encoding"),
            "base64".into(),
        );

        assert_eq!(
            headers.get::<ContentTransferEncoding>(),
            Some(ContentTransferEncoding::Base64)
        );
    }
}
