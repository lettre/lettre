use std::{
    io::{self, Write},
    ops::Deref,
};

use crate::message::header::ContentTransferEncoding;

/// A [`Message`][super::Message] or [`SinglePart`][super::SinglePart] body that has already been encoded.
#[derive(Debug, Clone)]
pub struct Body {
    buf: Vec<u8>,
    encoding: ContentTransferEncoding,
}

/// Either a `Vec<u8>` or a `String`.
///
/// If the content is valid utf-8 a `String` should be passed, as it
/// makes for a more efficient `Content-Transfer-Encoding` to be chosen.
#[derive(Debug, Clone)]
pub enum MaybeString {
    Binary(Vec<u8>),
    String(String),
}

impl Body {
    /// Encode the supplied `buf`, making it ready to be sent as a body.
    ///
    /// Takes a `Vec<u8>` or a `String`.
    ///
    /// Automatically chooses the most efficient encoding between
    /// `7bit`, `quoted-printable` and `base64`.
    ///
    /// If `buf` is valid utf-8 a `String` should be supplied, as `String`s
    /// can be encoded as `7bit` or `quoted-printable`, while `Vec<u8>` always
    /// get encoded as `base64`.
    pub fn new<B: Into<MaybeString>>(buf: B) -> Self {
        let buf: MaybeString = buf.into();

        let encoding = buf.encoding();
        Self::new_impl(buf.into(), encoding)
    }

    /// Encode the supplied `buf`, using the provided `encoding`.
    ///
    /// [`Body::new`] is generally the better option.
    ///
    /// Returns an [`Err`] giving back the supplied `buf`, in case the chosen
    /// encoding would have resulted into `buf` being encoded
    /// into an invalid body.
    pub fn new_with_encoding<B: Into<MaybeString>>(
        buf: B,
        encoding: ContentTransferEncoding,
    ) -> Result<Self, Vec<u8>> {
        let buf: MaybeString = buf.into();

        if !buf.is_encoding_ok(encoding) {
            return Err(buf.into());
        }

        Ok(Self::new_impl(buf.into(), encoding))
    }

    /// Builds a new `Body` using a pre-encoded buffer.
    ///
    /// **Generally not you want.**
    ///
    /// `buf` shouldn't contain non-ascii characters, lines longer than 1000 characters or nul bytes.
    #[inline]
    pub fn dangerous_pre_encoded(buf: Vec<u8>, encoding: ContentTransferEncoding) -> Self {
        Self { buf, encoding }
    }

    /// Encodes the supplied `buf` using the provided `encoding`
    fn new_impl(buf: Vec<u8>, encoding: ContentTransferEncoding) -> Self {
        match encoding {
            ContentTransferEncoding::SevenBit
            | ContentTransferEncoding::EightBit
            | ContentTransferEncoding::Binary => Self { buf, encoding },
            ContentTransferEncoding::QuotedPrintable => {
                let encoded = quoted_printable::encode(buf);

                Self::dangerous_pre_encoded(encoded, ContentTransferEncoding::QuotedPrintable)
            }
            ContentTransferEncoding::Base64 => {
                let base64_len = buf.len() * 4 / 3 + 4;
                let base64_endings_len = base64_len + base64_len / LINE_MAX_LENGTH;

                let mut out = Vec::with_capacity(base64_endings_len);
                {
                    let writer = LineWrappingWriter::new(&mut out, LINE_MAX_LENGTH);
                    let mut writer = base64::write::EncoderWriter::new(writer, base64::STANDARD);

                    // TODO: use writer.write_all(self.as_ref()).expect("base64 encoding never fails");

                    // modified Write::write_all to work around base64 crate bug
                    // TODO: remove once https://github.com/marshallpierce/rust-base64/issues/148 is fixed
                    {
                        let mut buf: &[u8] = buf.as_ref();
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

                Self::dangerous_pre_encoded(out, ContentTransferEncoding::Base64)
            }
        }
    }

    /// Returns the length of this `Body` in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if this `Body` has a length of zero, `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Returns the `Content-Transfer-Encoding` of this `Body`.
    #[inline]
    pub fn encoding(&self) -> ContentTransferEncoding {
        self.encoding
    }

    /// Consumes `Body` and returns the inner `Vec<u8>`
    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }
}

impl MaybeString {
    /// Suggests the best `Content-Transfer-Encoding` to be used for this `MaybeString`
    ///
    /// If the `MaybeString` was created from a `String` composed only of US-ASCII
    /// characters, with no lines longer than 1000 characters, then 7bit
    /// encoding will be used, else quoted-printable will be chosen.
    ///
    /// If the `MaybeString` was instead created from a `Vec<u8>`, base64 encoding is always
    /// chosen.
    ///
    /// `8bit` and `binary` encodings are never returned, as they may not be
    /// supported by all SMTP servers.
    pub fn encoding(&self) -> ContentTransferEncoding {
        match &self {
            Self::String(s) if is_7bit_encoded(s.as_ref()) => ContentTransferEncoding::SevenBit,
            // TODO: consider when base64 would be a better option because of output size
            Self::String(_) => ContentTransferEncoding::QuotedPrintable,
            Self::Binary(_) => ContentTransferEncoding::Base64,
        }
    }

    /// Returns `true` if using `encoding` to encode this `MaybeString`
    /// would result into an invalid encoded body.
    fn is_encoding_ok(&self, encoding: ContentTransferEncoding) -> bool {
        match encoding {
            ContentTransferEncoding::SevenBit => is_7bit_encoded(&self),
            ContentTransferEncoding::EightBit => is_8bit_encoded(&self),
            ContentTransferEncoding::Binary
            | ContentTransferEncoding::QuotedPrintable
            | ContentTransferEncoding::Base64 => true,
        }
    }
}

/// A trait for something that takes an encoded [`Body`].
///
/// Used by [`MessageBuilder::body`][super::MessageBuilder::body] and
/// [`SinglePartBuilder::body`][super::SinglePartBuilder::body],
/// which can either take something that can be encoded into [`Body`]
/// or a pre-encoded [`Body`].
///
/// If `encoding` is `None` the best encoding between `7bit`, `quoted-printable`
/// and `base64` is chosen based on the input body. **Best option.**
///
/// If `encoding` is `Some` the supplied encoding is used.
/// **NOTE:** if using the specified `encoding` would result into a malformed
/// body, this will panic!
pub trait IntoBody {
    fn into_body(self, encoding: Option<ContentTransferEncoding>) -> Body;
}

impl<T> IntoBody for T
where
    T: Into<MaybeString>,
{
    fn into_body(self, encoding: Option<ContentTransferEncoding>) -> Body {
        match encoding {
            Some(encoding) => Body::new_with_encoding(self, encoding).expect("invalid encoding"),
            None => Body::new(self),
        }
    }
}

impl IntoBody for Body {
    fn into_body(self, encoding: Option<ContentTransferEncoding>) -> Body {
        let _ = encoding;

        self
    }
}

impl AsRef<[u8]> for Body {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buf.as_ref()
    }
}

impl From<Vec<u8>> for MaybeString {
    #[inline]
    fn from(b: Vec<u8>) -> Self {
        Self::Binary(b)
    }
}

impl From<String> for MaybeString {
    #[inline]
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<MaybeString> for Vec<u8> {
    #[inline]
    fn from(s: MaybeString) -> Self {
        match s {
            MaybeString::Binary(b) => b,
            MaybeString::String(s) => s.into(),
        }
    }
}

impl Deref for MaybeString {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Binary(b) => b.as_ref(),
            Self::String(s) => s.as_ref(),
        }
    }
}

/// Checks whether it contains only US-ASCII characters,
/// and no lines are longer than 1000 characters including the `\n` character.
///
/// Most efficient content encoding available
fn is_7bit_encoded(buf: &[u8]) -> bool {
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
        let encoded = Body::new(String::from("Hello, world!"));

        assert_eq!(encoded.encoding(), ContentTransferEncoding::SevenBit);
        assert_eq!(encoded.as_ref(), b"Hello, world!");
    }

    #[test]
    fn seven_bit_encode() {
        let encoded = Body::new_with_encoding(
            String::from("Hello, world!"),
            ContentTransferEncoding::SevenBit,
        )
        .unwrap();

        assert_eq!(encoded.encoding(), ContentTransferEncoding::SevenBit);
        assert_eq!(encoded.as_ref(), b"Hello, world!");
    }

    #[test]
    fn seven_bit_too_long_detect() {
        let encoded = Body::new("Hello, world!".repeat(100));

        assert_eq!(encoded.encoding(), ContentTransferEncoding::QuotedPrintable);
        assert_eq!(
            encoded.as_ref(),
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
    fn seven_bit_too_long_fail() {
        let result = Body::new_with_encoding(
            "Hello, world!".repeat(100),
            ContentTransferEncoding::SevenBit,
        );

        assert!(result.is_err());
    }

    #[test]
    fn seven_bit_too_long_encode_quotedprintable() {
        let encoded = Body::new_with_encoding(
            "Hello, world!".repeat(100),
            ContentTransferEncoding::QuotedPrintable,
        )
        .unwrap();

        assert_eq!(encoded.encoding(), ContentTransferEncoding::QuotedPrintable);
        assert_eq!(
            encoded.as_ref(),
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
    fn seven_bit_invalid() {
        let result = Body::new_with_encoding(
            String::from("Привет, мир!"),
            ContentTransferEncoding::SevenBit,
        );

        assert!(result.is_err());
    }

    #[test]
    fn eight_bit_encode() {
        let encoded = Body::new_with_encoding(
            String::from("Привет, мир!"),
            ContentTransferEncoding::EightBit,
        )
        .unwrap();

        assert_eq!(encoded.encoding(), ContentTransferEncoding::EightBit);
        assert_eq!(encoded.as_ref(), "Привет, мир!".as_bytes());
    }

    #[test]
    fn eight_bit_too_long_fail() {
        let result = Body::new_with_encoding(
            "Привет, мир!".repeat(200),
            ContentTransferEncoding::EightBit,
        );

        assert!(result.is_err());
    }

    #[test]
    fn quoted_printable_detect() {
        let encoded = Body::new(String::from("Привет, мир!"));

        assert_eq!(encoded.encoding(), ContentTransferEncoding::QuotedPrintable);
        assert_eq!(
            encoded.as_ref(),
            b"=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!".as_ref()
        );
    }

    #[test]
    fn quoted_printable_encode_ascii() {
        let encoded = Body::new_with_encoding(
            String::from("Hello, world!"),
            ContentTransferEncoding::QuotedPrintable,
        )
        .unwrap();

        assert_eq!(encoded.encoding(), ContentTransferEncoding::QuotedPrintable);
        assert_eq!(encoded.as_ref(), b"Hello, world!");
    }

    #[test]
    fn quoted_printable_encode_utf8() {
        let encoded = Body::new_with_encoding(
            String::from("Привет, мир!"),
            ContentTransferEncoding::QuotedPrintable,
        )
        .unwrap();

        assert_eq!(encoded.encoding(), ContentTransferEncoding::QuotedPrintable);
        assert_eq!(
            encoded.as_ref(),
            b"=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!".as_ref()
        );
    }

    #[test]
    fn quoted_printable_encode_line_wrap() {
        let encoded = Body::new(String::from("Текст письма в уникоде"));

        assert_eq!(encoded.encoding(), ContentTransferEncoding::QuotedPrintable);
        assert_eq!(
            encoded.as_ref(),
            concat!(
                "=D0=A2=D0=B5=D0=BA=D1=81=D1=82 =D0=BF=D0=B8=D1=81=D1=8C=D0=BC=D0=B0 =D0=B2 =\r\n",
                "=D1=83=D0=BD=D0=B8=D0=BA=D0=BE=D0=B4=D0=B5"
            )
            .as_bytes()
        );
    }

    #[test]
    fn base64_detect() {
        let input = Body::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let encoding = input.encoding();
        assert_eq!(encoding, ContentTransferEncoding::Base64);
    }

    #[test]
    fn base64_encode_bytes() {
        let encoded = Body::new_with_encoding(
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            ContentTransferEncoding::Base64,
        )
        .unwrap();

        assert_eq!(encoded.encoding(), ContentTransferEncoding::Base64);
        assert_eq!(encoded.as_ref(), b"AAECAwQFBgcICQ==");
    }

    #[test]
    fn base64_encode_bytes_wrapping() {
        let encoded = Body::new_with_encoding(
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9].repeat(20),
            ContentTransferEncoding::Base64,
        )
        .unwrap();

        assert_eq!(encoded.encoding(), ContentTransferEncoding::Base64);
        assert_eq!(
            encoded.as_ref(),
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
        let encoded = Body::new_with_encoding(
            String::from("Hello World!"),
            ContentTransferEncoding::Base64,
        )
        .unwrap();

        assert_eq!(encoded.encoding(), ContentTransferEncoding::Base64);
        assert_eq!(encoded.as_ref(), b"SGVsbG8gV29ybGQh");
    }

    #[test]
    fn base64_encode_ascii_wrapping() {
        let encoded =
            Body::new_with_encoding("Hello World!".repeat(20), ContentTransferEncoding::Base64)
                .unwrap();

        assert_eq!(encoded.encoding(), ContentTransferEncoding::Base64);
        assert_eq!(
            encoded.as_ref(),
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
