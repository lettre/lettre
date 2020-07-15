use crate::message::header::ContentTransferEncoding;
use line_wrap::{crlf, line_wrap, LineEnding};
use std::io::Write;

/// Encoder trait
pub trait EncoderCodec: Send {
    /// Encode all data
    fn encode(&mut self, input: &[u8]) -> Vec<u8>;
}

/// 7bit codec
///
/// WARNING: Panics when passed non-ascii chars
struct SevenBitCodec {
    line_wrapper: EightBitCodec,
}

impl SevenBitCodec {
    pub fn new() -> Self {
        SevenBitCodec {
            line_wrapper: EightBitCodec::new(),
        }
    }
}

impl EncoderCodec for SevenBitCodec {
    fn encode(&mut self, input: &[u8]) -> Vec<u8> {
        if input.iter().all(u8::is_ascii) {
            self.line_wrapper.encode(input)
        } else {
            panic!("")
        }
    }
}

/// Quoted-Printable codec
///
struct QuotedPrintableCodec();

impl QuotedPrintableCodec {
    pub fn new() -> Self {
        QuotedPrintableCodec()
    }
}

impl EncoderCodec for QuotedPrintableCodec {
    fn encode(&mut self, input: &[u8]) -> Vec<u8> {
        quoted_printable::encode(input)
    }
}

/// Base64 codec
///
struct Base64Codec {
    line_wrapper: EightBitCodec,
}

impl Base64Codec {
    pub fn new() -> Self {
        Base64Codec {
            // TODO probably 78, 76 is for qp
            line_wrapper: EightBitCodec::new().with_limit(78 - 2),
        }
    }
}

impl EncoderCodec for Base64Codec {
    fn encode(&mut self, input: &[u8]) -> Vec<u8> {
        self.line_wrapper.encode(base64::encode(input).as_bytes())
    }
}

/// 8bit codec
///
struct EightBitCodec {
    max_length: usize,
}

const DEFAULT_MAX_LINE_LENGTH: usize = 1000 - 2;

impl EightBitCodec {
    pub fn new() -> Self {
        EightBitCodec {
            max_length: DEFAULT_MAX_LINE_LENGTH,
        }
    }

    pub fn with_limit(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }
}

impl EncoderCodec for EightBitCodec {
    fn encode(&mut self, input: &[u8]) -> Vec<u8> {
        let ending = &crlf();

        let mut out = vec![0_u8; input.len() + input.len() / self.max_length * ending.len()];
        let mut writer: &mut [u8] = out.as_mut();
        writer.write_all(input).unwrap();

        line_wrap(&mut out, input.len(), self.max_length, ending);
        out
    }
}

/// Binary codec
///
struct BinaryCodec;

impl BinaryCodec {
    pub fn new() -> Self {
        BinaryCodec
    }
}

impl EncoderCodec for BinaryCodec {
    fn encode(&mut self, input: &[u8]) -> Vec<u8> {
        input.into()
    }
}

pub fn codec(encoding: Option<&ContentTransferEncoding>) -> Box<dyn EncoderCodec> {
    use self::ContentTransferEncoding::*;
    if let Some(encoding) = encoding {
        match encoding {
            SevenBit => Box::new(SevenBitCodec::new()),
            QuotedPrintable => Box::new(QuotedPrintableCodec::new()),
            Base64 => Box::new(Base64Codec::new()),
            EightBit => Box::new(EightBitCodec::new()),
            Binary => Box::new(BinaryCodec::new()),
        }
    } else {
        Box::new(BinaryCodec::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn seven_bit_encode() {
        let mut c = SevenBitCodec::new();

        assert_eq!(
            &String::from_utf8(c.encode("Hello, world!".as_bytes())).unwrap(),
            "Hello, world!"
        );
    }

    #[test]
    #[should_panic]
    fn seven_bit_encode_panic() {
        let mut c = SevenBitCodec::new();
        c.encode("Hello, мир!".as_bytes());
    }

    #[test]
    fn quoted_printable_encode() {
        let mut c = QuotedPrintableCodec::new();

        assert_eq!(
            &String::from_utf8(c.encode("Привет, мир!".as_bytes())).unwrap(),
            "=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!"
        );

        assert_eq!(&String::from_utf8(c.encode("Текст письма в уникоде".as_bytes())).unwrap(),
                   "=D0=A2=D0=B5=D0=BA=D1=81=D1=82 =D0=BF=D0=B8=D1=81=D1=8C=D0=BC=D0=B0 =D0=B2 =\r\n=D1=83=D0=BD=D0=B8=D0=BA=D0=BE=D0=B4=D0=B5");
    }

    #[test]
    fn base64_encode() {
        let mut c = Base64Codec::new();

        assert_eq!(
            &String::from_utf8(c.encode("Привет, мир!".as_bytes())).unwrap(),
            "0J/RgNC40LLQtdGCLCDQvNC40YAh"
        );

        assert_eq!(
            &String::from_utf8(c.encode("Текст письма в уникоде подлиннее.".as_bytes())).unwrap(),
            concat!(
                "0KLQtdC60YHRgiDQv9C40YHRjNC80LAg0LIg0YPQvdC40LrQ",
                "vtC00LUg0L/QvtC00LvQuNC90L3Q\r\ntdC1Lg=="
            )
        );

        assert_eq!(
            &String::from_utf8(c.encode(
                "Ну прямо супер-длинный текст письма в уникоде, который уж точно ну никак не поместиться в 78 байт, как ни крути, я гарантирую.".as_bytes()
                    )).unwrap(),

                concat!("0J3RgyDQv9GA0Y/QvNC+INGB0YPQv9C10YAt0LTQu9C40L3QvdGL0Lkg0YLQtdC60YHRgiDQv9C4\r\n",
                        "0YHRjNC80LAg0LIg0YPQvdC40LrQvtC00LUsINC60L7RgtC+0YDRi9C5INGD0LYg0YLQvtGH0L3Q\r\n",
                        "viDQvdGDINC90LjQutCw0Log0L3QtSDQv9C+0LzQtdGB0YLQuNGC0YzRgdGPINCyIDc4INCx0LDQ\r\n",
                        "udGCLCDQutCw0Log0L3QuCDQutGA0YPRgtC4LCDRjyDQs9Cw0YDQsNC90YLQuNGA0YPRji4=")
        );
        assert_eq!(
            &String::from_utf8(c.encode(
                "Ну прямо супер-длинный текст письма в уникоде, который уж точно ну никак не поместиться в 78 байт, как ни крути, я гарантирую это.".as_bytes()
            )).unwrap(),

                concat!("0J3RgyDQv9GA0Y/QvNC+INGB0YPQv9C10YAt0LTQu9C40L3QvdGL0Lkg0YLQtdC60YHRgiDQv9C4\r\n",
                        "0YHRjNC80LAg0LIg0YPQvdC40LrQvtC00LUsINC60L7RgtC+0YDRi9C5INGD0LYg0YLQvtGH0L3Q\r\n",
                        "viDQvdGDINC90LjQutCw0Log0L3QtSDQv9C+0LzQtdGB0YLQuNGC0YzRgdGPINCyIDc4INCx0LDQ\r\n",
                        "udGCLCDQutCw0Log0L3QuCDQutGA0YPRgtC4LCDRjyDQs9Cw0YDQsNC90YLQuNGA0YPRjiDRjdGC\r\n",
                        "0L4u")
        );
    }

    #[test]
    fn base64_encodeed() {
        let mut c = Base64Codec::new();

        assert_eq!(
            &String::from_utf8(c.encode("Chunk.".as_bytes())).unwrap(),
            "Q2h1bmsu"
        );
    }

    #[test]
    fn eight_bit_encode() {
        let mut c = EightBitCodec::new();

        assert_eq!(
            &String::from_utf8(c.encode("Hello, world!".as_bytes())).unwrap(),
            "Hello, world!"
        );

        assert_eq!(
            &String::from_utf8(c.encode("Hello, мир!".as_bytes())).unwrap(),
            "Hello, мир!"
        );
    }

    #[test]
    fn binary_encode() {
        let mut c = BinaryCodec::new();

        assert_eq!(
            &String::from_utf8(c.encode("Hello, world!".as_bytes())).unwrap(),
            "Hello, world!"
        );

        assert_eq!(
            &String::from_utf8(c.encode("Hello, мир!".as_bytes())).unwrap(),
            "Hello, мир!"
        );
    }
}
