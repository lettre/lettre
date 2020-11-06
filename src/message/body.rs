use std::borrow::Cow;
use std::io::{self, Write};

use crate::message::header::ContentTransferEncoding;

/// A [`SinglePart`][super::SinglePart] body.
#[derive(Debug, Clone)]
pub struct Body(BodyInner);

#[derive(Debug, Clone)]
enum BodyInner {
    Binary(Vec<u8>),
    String(String),
}

impl Body {
    /// Returns the length of this `Body` in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        match &self.0 {
            BodyInner::Binary(b) => b.len(),
            BodyInner::String(s) => s.len(),
        }
    }

    /// Returns `true` if this `Body` has a length of zero, `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        match &self.0 {
            BodyInner::Binary(b) => b.is_empty(),
            BodyInner::String(s) => s.is_empty(),
        }
    }

    /// Suggests the best `Content-Transfer-Encoding` to be used for this `Body`
    ///
    /// If the `Body` was created from a `String` composed only of US-ASCII
    /// characters, with no lines longer than 1000 characters, then 7bit
    /// encoding will be used, else quoted-printable will be choosen.
    ///
    /// If the `Body` was instead created from a `Vec<u8>`, base64 encoding is always
    /// choosen.
    ///
    /// `8bit` and `binary` encodings are never returned, as they may not be
    /// supported by all SMTP servers.
    pub fn encoding(&self) -> ContentTransferEncoding {
        match &self.0 {
            BodyInner::String(s) if is_7bit_encoded(s.as_ref()) => {
                ContentTransferEncoding::SevenBit
            }
            // TODO: consider when base64 would be a better option because of output size
            BodyInner::String(_) => ContentTransferEncoding::QuotedPrintable,
            BodyInner::Binary(_) => ContentTransferEncoding::Base64,
        }
    }

    /// Encodes this `Body` using the choosen `encoding`.
    ///
    /// # Panic
    ///
    /// Panics if the choosen `Content-Transfer-Encoding` would end-up
    /// creating an incorrectly encoded email.
    ///
    /// Could happen for example if `7bit` encoding is choosen when the
    /// content isn't US-ASCII or contains lines longer than 1000 characters.
    ///
    /// Never panics when using an `encoding` returned by [`encoding`][Body::encoding].
    pub fn encode(&self, encoding: ContentTransferEncoding) -> Cow<'_, Body> {
        match encoding {
            ContentTransferEncoding::SevenBit => {
                assert!(
                    is_7bit_encoded(self.as_ref()),
                    "Body isn't valid 7bit content"
                );

                Cow::Borrowed(self)
            }
            ContentTransferEncoding::EightBit => {
                assert!(
                    is_8bit_encoded(self.as_ref()),
                    "Body isn't valid 8bit content"
                );

                Cow::Borrowed(self)
            }
            ContentTransferEncoding::Binary => Cow::Borrowed(self),
            ContentTransferEncoding::QuotedPrintable => {
                let encoded = quoted_printable::encode_to_str(self);
                Cow::Owned(Body(BodyInner::String(encoded)))
            }
            ContentTransferEncoding::Base64 => {
                let base64_len = self.len() * 4 / 3 + 4;
                let base64_endings_len = base64_len + base64_len / LINE_MAX_LENGTH;

                let mut out = Vec::with_capacity(base64_endings_len);
                {
                    let writer = LineWrappingWriter::new(&mut out, LINE_MAX_LENGTH);
                    let mut writer = base64::write::EncoderWriter::new(writer, base64::STANDARD);

                    // TODO: use writer.write_all(self.as_ref()).expect("base64 encoding never fails");

                    // modified Write::write_all to work around base64 crate bug
                    // TODO: remove once https://github.com/marshallpierce/rust-base64/issues/148 is fixed
                    {
                        let mut buf: &[u8] = self.as_ref();
                        while !buf.is_empty() {
                            match writer.write(buf) {
                                Ok(0) => {
                                    // ignore 0 writes
                                }
                                Ok(n) => {
                                    buf = &buf[n..];
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                                Err(e) => panic!("base64 encoding never fails: {}", e),
                            }
                        }
                    }
                }

                Cow::Owned(Body(BodyInner::Binary(out)))
            }
        }
    }
}

impl From<Vec<u8>> for Body {
    #[inline]
    fn from(b: Vec<u8>) -> Self {
        Self(BodyInner::Binary(b))
    }
}

impl From<String> for Body {
    #[inline]
    fn from(s: String) -> Self {
        Self(BodyInner::String(s))
    }
}

impl AsRef<[u8]> for Body {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match &self.0 {
            BodyInner::Binary(b) => b.as_ref(),
            BodyInner::String(s) => s.as_ref(),
        }
    }
}

/// Checks whether it contains only US-ASCII characters,
/// and no lines are longer than 1000 characters including the `\n` character.
///
/// Most efficient content encoding available
pub(crate) fn is_7bit_encoded(buf: &[u8]) -> bool {
    buf.is_ascii() && !contains_too_long_lines(buf)
}

/// Checks that no lines are longer than 1000 characters,
/// including the `\n` character.
/// NOTE: 8bit isn't supported by all SMTP servers.
fn is_8bit_encoded(buf: &[u8]) -> bool {
    !contains_too_long_lines(buf)
}

/// Checks if there are lines that are longer than 1000 characters,
/// including the `\n` character.
fn contains_too_long_lines(buf: &[u8]) -> bool {
    buf.len() > 1000 && buf.split(|&b| b == b'\n').any(|line| line.len() > 999)
}

const LINE_SEPARATOR: &[u8] = b"\r\n";
const LINE_MAX_LENGTH: usize = 78 - LINE_SEPARATOR.len();

/// A `Write`r that inserts a line separator `\r\n` every `max_line_length` bytes.
struct LineWrappingWriter<'a, W> {
    writer: &'a mut W,
    current_line_length: usize,
    max_line_length: usize,
}

impl<'a, W> LineWrappingWriter<'a, W> {
    pub fn new(writer: &'a mut W, max_line_length: usize) -> Self {
        Self {
            writer,
            current_line_length: 0,
            max_line_length,
        }
    }
}

impl<'a, W> Write for LineWrappingWriter<'a, W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let remaining_line_len = self.max_line_length - self.current_line_length;
        let write_len = std::cmp::min(buf.len(), remaining_line_len);

        self.writer.write_all(&buf[..write_len])?;

        if remaining_line_len == write_len {
            self.writer.write_all(LINE_SEPARATOR)?;

            self.current_line_length = 0;
        } else {
            self.current_line_length += write_len;
        }

        Ok(write_len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
mod test {
    use super::{Body, ContentTransferEncoding};

    #[test]
    fn seven_bit_detect() {
        let input = Body::from(String::from("Hello, world!"));

        let encoding = input.encoding();
        assert_eq!(encoding, ContentTransferEncoding::SevenBit);
    }

    #[test]
    fn seven_bit_encode() {
        let input = Body::from(String::from("Hello, world!"));

        let output = input.encode(ContentTransferEncoding::SevenBit);
        assert_eq!(output.as_ref().as_ref(), b"Hello, world!");
    }

    #[test]
    fn seven_bit_too_long_detect() {
        let input = Body::from("Hello, world!".repeat(100));

        let encoding = input.encoding();
        assert_eq!(encoding, ContentTransferEncoding::QuotedPrintable);
    }

    #[test]
    #[should_panic]
    fn seven_bit_too_long_fail() {
        let input = Body::from("Hello, world!".repeat(100));

        let _ = input.encode(ContentTransferEncoding::SevenBit);
    }

    #[test]
    fn seven_bit_too_long_encode_quotedprintable() {
        let input = Body::from("Hello, world!".repeat(100));

        let output = input.encode(ContentTransferEncoding::QuotedPrintable);
        assert_eq!(
            output.as_ref().as_ref(),
            concat!(
                "Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, wor=\r\n",
                "ld!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, =\r\n",
                "world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hell=\r\n",
                "o, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!H=\r\n",
                "ello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, worl=\r\n",
                "d!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, w=\r\n",
                "orld!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello=\r\n",
                ", world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!He=\r\n",
                "llo, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world=\r\n",
                "!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, wo=\r\n",
                "rld!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello,=\r\n",
                " world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hel=\r\n",
                "lo, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!=\r\n",
                "Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, wor=\r\n",
                "ld!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, =\r\n",
                "world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!Hell=\r\n",
                "o, world!Hello, world!Hello, world!Hello, world!Hello, world!Hello, world!H=\r\n",
                "ello, world!Hello, world!"
            )
            .as_bytes()
        );
    }

    #[test]
    #[should_panic]
    fn seven_bit_invalid() {
        let input = Body::from(String::from("Привет, мир!"));

        let _ = input.encode(ContentTransferEncoding::SevenBit);
    }

    #[test]
    fn eight_bit_encode() {
        let input = Body::from(String::from("Привет, мир!"));

        let out = input.encode(ContentTransferEncoding::EightBit);
        assert_eq!(out.as_ref().as_ref(), "Привет, мир!".as_bytes());
    }

    #[test]
    #[should_panic]
    fn eight_bit_too_long_fail() {
        let input = Body::from("Привет, мир!".repeat(200));

        let _ = input.encode(ContentTransferEncoding::EightBit);
    }

    #[test]
    fn quoted_printable_detect() {
        let input = Body::from(String::from("Привет, мир!"));

        let encoding = input.encoding();
        assert_eq!(encoding, ContentTransferEncoding::QuotedPrintable);
    }

    #[test]
    fn quoted_printable_encode_ascii() {
        let input = Body::from(String::from("Hello, world!"));

        let output = input.encode(ContentTransferEncoding::QuotedPrintable);
        assert_eq!(output.as_ref().as_ref(), b"Hello, world!");
    }

    #[test]
    fn quoted_printable_encode_utf8() {
        let input = Body::from(String::from("Привет, мир!"));

        let output = input.encode(ContentTransferEncoding::QuotedPrintable);
        assert_eq!(
            output.as_ref().as_ref(),
            b"=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!".as_ref()
        );
    }

    #[test]
    fn quoted_printable_encode_line_wrap() {
        let input = Body::from(String::from("Текст письма в уникоде"));

        let output = input.encode(ContentTransferEncoding::QuotedPrintable);
        assert_eq!(
            output.as_ref().as_ref(),
            concat!(
                "=D0=A2=D0=B5=D0=BA=D1=81=D1=82 =D0=BF=D0=B8=D1=81=D1=8C=D0=BC=D0=B0 =D0=B2 =\r\n",
                "=D1=83=D0=BD=D0=B8=D0=BA=D0=BE=D0=B4=D0=B5"
            )
            .as_bytes()
        );
    }

    #[test]
    fn base64_detect() {
        let input = Body::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let encoding = input.encoding();
        assert_eq!(encoding, ContentTransferEncoding::Base64);
    }

    #[test]
    fn base64_encode_bytes() {
        let input = Body::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let output = input.encode(ContentTransferEncoding::Base64);
        assert_eq!(output.as_ref().as_ref(), b"AAECAwQFBgcICQ==");
    }

    #[test]
    fn base64_encode_bytes_wrapping() {
        let input = Body::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9].repeat(20));

        let output = input.encode(ContentTransferEncoding::Base64);
        assert_eq!(
            output.as_ref().as_ref(),
            concat!(
                "AAECAwQFBgcICQABAgMEBQYHCAkAAQIDBAUGBwgJAAECAwQFBgcICQABAgMEBQYHCAkAAQIDBAUG\r\n",
                "BwgJAAECAwQFBgcICQABAgMEBQYHCAkAAQIDBAUGBwgJAAECAwQFBgcICQABAgMEBQYHCAkAAQID\r\n",
                "BAUGBwgJAAECAwQFBgcICQABAgMEBQYHCAkAAQIDBAUGBwgJAAECAwQFBgcICQABAgMEBQYHCAkA\r\n",
                "AQIDBAUGBwgJAAECAwQFBgcICQABAgMEBQYHCAk="
            )
            .as_bytes()
        );
    }

    #[test]
    fn base64_encode_ascii() {
        let input = Body::from(String::from("Hello World!"));

        let output = input.encode(ContentTransferEncoding::Base64);
        assert_eq!(output.as_ref().as_ref(), b"SGVsbG8gV29ybGQh");
    }

    #[test]
    fn base64_encode_ascii_wrapping() {
        let input = Body::from("Hello World!".repeat(20));

        let output = input.encode(ContentTransferEncoding::Base64);
        assert_eq!(
            output.as_ref().as_ref(),
            concat!(
                "SGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29y\r\n",
                "bGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8g\r\n",
                "V29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVs\r\n",
                "bG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQhSGVsbG8gV29ybGQh\r\n",
                "SGVsbG8gV29ybGQh"
            )
            .as_bytes()
        );
    }
}
