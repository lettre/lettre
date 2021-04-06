use std::io::Write;

use crate::message::{
    header::{ContentTransferEncoding, ContentType, Header, Headers},
    EmailFormat, IntoBody,
};
use mime::Mime;
use rand::Rng;

/// MIME part variants
#[derive(Debug, Clone)]
pub enum Part {
    /// Single part with content
    Single(SinglePart),

    /// Multiple parts of content
    Multi(MultiPart),
}

impl EmailFormat for Part {
    fn format(&self, out: &mut Vec<u8>) {
        match self {
            Part::Single(part) => part.format(out),
            Part::Multi(part) => part.format(out),
        }
    }
}

impl Part {
    /// Get message content formatted for SMTP
    pub fn formatted(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.format(&mut out);
        out
    }
}

/// Parts of multipart body
pub type Parts = Vec<Part>;

/// Creates builder for single part
#[derive(Debug, Clone)]
pub struct SinglePartBuilder {
    headers: Headers,
}

impl SinglePartBuilder {
    /// Creates a default singlepart builder
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
        }
    }

    /// Set the header to singlepart
    pub fn header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Set the Content-Type header of the singlepart
    pub fn content_type(mut self, content_type: ContentType) -> Self {
        self.headers.set(content_type);
        self
    }

    /// Build singlepart using body
    pub fn body<T: IntoBody>(mut self, body: T) -> SinglePart {
        let maybe_encoding = self.headers.get::<ContentTransferEncoding>().copied();
        let body = body.into_body(maybe_encoding);

        self.headers.set(body.encoding());

        SinglePart {
            headers: self.headers,
            body: body.into_vec(),
        }
    }
}

impl Default for SinglePartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Single part
///
/// # Example
///
/// ```
/// use lettre::message::{header, SinglePart};
///
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let part = SinglePart::builder()
///     .header(header::ContentType("text/plain; charset=utf8".parse()?))
///     .body(String::from("Текст письма в уникоде"));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SinglePart {
    headers: Headers,
    body: Vec<u8>,
}

impl SinglePart {
    /// Creates a builder for singlepart
    #[inline]
    pub fn builder() -> SinglePartBuilder {
        SinglePartBuilder::new()
    }

    /// Get the headers from singlepart
    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get the encoded body
    #[inline]
    pub fn raw_body(&self) -> &[u8] {
        &self.body
    }

    /// Get message content formatted for sending
    pub fn formatted(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.format(&mut out);
        out
    }
}

impl EmailFormat for SinglePart {
    fn format(&self, out: &mut Vec<u8>) {
        write!(out, "{}", self.headers)
            .expect("A Write implementation panicked while formatting headers");
        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(&self.body);
        out.extend_from_slice(b"\r\n");
    }
}

/// The kind of multipart
#[derive(Debug, Clone)]
pub enum MultiPartKind {
    /// Mixed kind to combine unrelated content parts
    ///
    /// For example this kind can be used to mix email message and attachments.
    Mixed,

    /// Alternative kind to join several variants of same email contents.
    ///
    /// That kind is recommended to use for joining plain (text) and rich (HTML) messages into single email message.
    Alternative,

    /// Related kind to mix content and related resources.
    ///
    /// For example, you can include images into HTML content using that.
    Related,

    /// Encrypted kind for encrypted messages
    Encrypted { protocol: String },

    /// Signed kind for signed messages
    Signed { protocol: String, micalg: String },
}

/// Create a random MIME boundary.
fn make_boundary() -> String {
    rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(40)
        .map(char::from)
        .collect()
}

impl MultiPartKind {
    fn to_mime<S: Into<String>>(&self, boundary: Option<S>) -> Mime {
        let boundary = boundary.map_or_else(make_boundary, Into::into);

        format!(
            "multipart/{}; boundary=\"{}\"{}",
            match self {
                Self::Mixed => "mixed",
                Self::Alternative => "alternative",
                Self::Related => "related",
                Self::Encrypted { .. } => "encrypted",
                Self::Signed { .. } => "signed",
            },
            boundary,
            match self {
                Self::Encrypted { protocol } => format!("; protocol=\"{}\"", protocol),
                Self::Signed { protocol, micalg } =>
                    format!("; protocol=\"{}\"; micalg=\"{}\"", protocol, micalg),
                _ => String::new(),
            }
        )
        .parse()
        .unwrap()
    }

    fn from_mime(m: &Mime) -> Option<Self> {
        match m.subtype().as_ref() {
            "mixed" => Some(Self::Mixed),
            "alternative" => Some(Self::Alternative),
            "related" => Some(Self::Related),
            "signed" => m.get_param("protocol").and_then(|p| {
                m.get_param("micalg").map(|micalg| Self::Signed {
                    protocol: p.as_str().to_owned(),
                    micalg: micalg.as_str().to_owned(),
                })
            }),
            "encrypted" => m.get_param("protocol").map(|p| Self::Encrypted {
                protocol: p.as_str().to_owned(),
            }),
            _ => None,
        }
    }
}

impl From<MultiPartKind> for Mime {
    fn from(m: MultiPartKind) -> Self {
        m.to_mime::<String>(None)
    }
}

/// Multipart builder
#[derive(Debug, Clone)]
pub struct MultiPartBuilder {
    headers: Headers,
}

impl MultiPartBuilder {
    /// Creates default multipart builder
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
        }
    }

    /// Set a header
    pub fn header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Set `Content-Type` header using [`MultiPartKind`]
    pub fn kind(self, kind: MultiPartKind) -> Self {
        self.header(ContentType(kind.into()))
    }

    /// Set custom boundary
    pub fn boundary<S: AsRef<str>>(self, boundary: S) -> Self {
        let kind = {
            let mime = &self.headers.get::<ContentType>().unwrap().0;
            MultiPartKind::from_mime(mime).unwrap()
        };
        let mime = kind.to_mime(Some(boundary.as_ref()));
        self.header(ContentType(mime))
    }

    /// Creates multipart without parts
    pub fn build(self) -> MultiPart {
        MultiPart {
            headers: self.headers,
            parts: Vec::new(),
        }
    }

    /// Creates multipart using part
    pub fn part(self, part: Part) -> MultiPart {
        self.build().part(part)
    }

    /// Creates multipart using singlepart
    pub fn singlepart(self, part: SinglePart) -> MultiPart {
        self.build().singlepart(part)
    }

    /// Creates multipart using multipart
    pub fn multipart(self, part: MultiPart) -> MultiPart {
        self.build().multipart(part)
    }
}

impl Default for MultiPartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Multipart variant with parts
#[derive(Debug, Clone)]
pub struct MultiPart {
    headers: Headers,
    parts: Parts,
}

impl MultiPart {
    /// Creates multipart builder
    pub fn builder() -> MultiPartBuilder {
        MultiPartBuilder::new()
    }

    /// Creates mixed multipart builder
    ///
    /// Shortcut for `MultiPart::builder().kind(MultiPartKind::Mixed)`
    pub fn mixed() -> MultiPartBuilder {
        MultiPart::builder().kind(MultiPartKind::Mixed)
    }

    /// Creates alternative multipart builder
    ///
    /// Shortcut for `MultiPart::builder().kind(MultiPartKind::Alternative)`
    pub fn alternative() -> MultiPartBuilder {
        MultiPart::builder().kind(MultiPartKind::Alternative)
    }

    /// Creates related multipart builder
    ///
    /// Shortcut for `MultiPart::builder().kind(MultiPartKind::Related)`
    pub fn related() -> MultiPartBuilder {
        MultiPart::builder().kind(MultiPartKind::Related)
    }

    /// Creates encrypted multipart builder
    ///
    /// Shortcut for `MultiPart::builder().kind(MultiPartKind::Encrypted{ protocol })`
    pub fn encrypted(protocol: String) -> MultiPartBuilder {
        MultiPart::builder().kind(MultiPartKind::Encrypted { protocol })
    }

    /// Creates signed multipart builder
    ///
    /// Shortcut for `MultiPart::builder().kind(MultiPartKind::Signed{ protocol, micalg })`
    pub fn signed(protocol: String, micalg: String) -> MultiPartBuilder {
        MultiPart::builder().kind(MultiPartKind::Signed { protocol, micalg })
    }

    /// Add part to multipart
    pub fn part(mut self, part: Part) -> Self {
        self.parts.push(part);
        self
    }

    /// Add single part to multipart
    pub fn singlepart(mut self, part: SinglePart) -> Self {
        self.parts.push(Part::Single(part));
        self
    }

    /// Add multi part to multipart
    pub fn multipart(mut self, part: MultiPart) -> Self {
        self.parts.push(Part::Multi(part));
        self
    }

    /// Get the boundary of multipart contents
    pub fn boundary(&self) -> String {
        let content_type = &self.headers.get::<ContentType>().unwrap().0;
        content_type.get_param("boundary").unwrap().as_str().into()
    }

    /// Get the headers from the multipart
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the headers
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Get the parts from the multipart
    pub fn parts(&self) -> &Parts {
        &self.parts
    }

    /// Get a mutable reference to the parts
    pub fn parts_mut(&mut self) -> &mut Parts {
        &mut self.parts
    }

    /// Get message content formatted for SMTP
    pub fn formatted(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.format(&mut out);
        out
    }
}

impl EmailFormat for MultiPart {
    fn format(&self, out: &mut Vec<u8>) {
        write!(out, "{}", self.headers)
            .expect("A Write implementation panicked while formatting headers");
        out.extend_from_slice(b"\r\n");

        let boundary = self.boundary();

        for part in &self.parts {
            out.extend_from_slice(b"--");
            out.extend_from_slice(boundary.as_bytes());
            out.extend_from_slice(b"\r\n");
            part.format(out);
        }

        out.extend_from_slice(b"--");
        out.extend_from_slice(boundary.as_bytes());
        out.extend_from_slice(b"--\r\n");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::message::header;

    #[test]
    fn single_part_binary() {
        let part = SinglePart::builder()
            .header(header::ContentType(
                "text/plain; charset=utf8".parse().unwrap(),
            ))
            .header(header::ContentTransferEncoding::Binary)
            .body(String::from("Текст письма в уникоде"));

        assert_eq!(
            String::from_utf8(part.formatted()).unwrap(),
            concat!(
                "Content-Type: text/plain; charset=utf8\r\n",
                "Content-Transfer-Encoding: binary\r\n",
                "\r\n",
                "Текст письма в уникоде\r\n"
            )
        );
    }

    #[test]
    fn single_part_quoted_printable() {
        let part = SinglePart::builder()
            .header(header::ContentType(
                "text/plain; charset=utf8".parse().unwrap(),
            ))
            .header(header::ContentTransferEncoding::QuotedPrintable)
            .body(String::from("Текст письма в уникоде"));

        assert_eq!(
            String::from_utf8(part.formatted()).unwrap(),
            concat!(
                "Content-Type: text/plain; charset=utf8\r\n",
                "Content-Transfer-Encoding: quoted-printable\r\n",
                "\r\n",
                "=D0=A2=D0=B5=D0=BA=D1=81=D1=82 =D0=BF=D0=B8=D1=81=D1=8C=D0=BC=D0=B0 =D0=B2 =\r\n",
                "=D1=83=D0=BD=D0=B8=D0=BA=D0=BE=D0=B4=D0=B5\r\n"
            )
        );
    }

    #[test]
    fn single_part_base64() {
        let part = SinglePart::builder()
            .header(header::ContentType(
                "text/plain; charset=utf8".parse().unwrap(),
            ))
            .header(header::ContentTransferEncoding::Base64)
            .body(String::from("Текст письма в уникоде"));

        assert_eq!(
            String::from_utf8(part.formatted()).unwrap(),
            concat!(
                "Content-Type: text/plain; charset=utf8\r\n",
                "Content-Transfer-Encoding: base64\r\n",
                "\r\n",
                "0KLQtdC60YHRgiDQv9C40YHRjNC80LAg0LIg0YPQvdC40LrQvtC00LU=\r\n"
            )
        );
    }

    #[test]
    fn multi_part_mixed() {
        let part = MultiPart::mixed()
            .boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
            .part(Part::Single(
                SinglePart::builder()
                    .header(header::ContentType(
                        "text/plain; charset=utf8".parse().unwrap(),
                    ))
                    .header(header::ContentTransferEncoding::Binary)
                    .body(String::from("Текст письма в уникоде")),
            ))
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType(
                        "text/plain; charset=utf8".parse().unwrap(),
                    ))
                    .header(header::ContentDisposition {
                        disposition: header::DispositionType::Attachment,
                        parameters: vec![header::DispositionParam::Filename(
                            header::Charset::Ext("utf-8".into()),
                            None,
                            "example.c".into(),
                        )],
                    })
                    .header(header::ContentTransferEncoding::Binary)
                    .body(String::from("int main() { return 0; }")),
            );

        assert_eq!(String::from_utf8(part.formatted()).unwrap(),
                   concat!("Content-Type: multipart/mixed;",
                           " boundary=\"F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\"\r\n",
                           "\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "Текст письма в уникоде\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Disposition: attachment; filename=\"example.c\"\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "int main() { return 0; }\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK--\r\n"));
    }
    #[test]
    fn multi_part_encrypted() {
        let part = MultiPart::encrypted("application/pgp-encrypted".to_owned())
            .boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
            .part(Part::Single(
                SinglePart::builder()
                    .header(header::ContentType(
                        "application/pgp-encrypted".parse().unwrap(),
                    ))
                    .body(String::from("Version: 1")),
            ))
            .singlepart(
                SinglePart::builder()
                    .header(ContentType(
                        "application/octet-stream; name=\"encrypted.asc\""
                            .parse()
                            .unwrap(),
                    ))
                    .header(header::ContentDisposition {
                        disposition: header::DispositionType::Inline,
                        parameters: vec![header::DispositionParam::Filename(
                            header::Charset::Ext("utf-8".into()),
                            None,
                            "encrypted.asc".into(),
                        )],
                    })
                    .body(String::from(concat!(
                        "-----BEGIN PGP MESSAGE-----\r\n",
                        "wV4D0dz5vDXklO8SAQdA5lGX1UU/eVQqDxNYdHa7tukoingHzqUB6wQssbMfHl8w\r\n",
                        "...\r\n",
                        "-----END PGP MESSAGE-----\r\n"
                    ))),
            );

        assert_eq!(String::from_utf8(part.formatted()).unwrap(),
                   concat!("Content-Type: multipart/encrypted;",
                           " boundary=\"F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\";",
                           " protocol=\"application/pgp-encrypted\"\r\n",
                           "\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: application/pgp-encrypted\r\n",
                           "Content-Transfer-Encoding: 7bit\r\n",
                           "\r\n",
                           "Version: 1\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: application/octet-stream; name=\"encrypted.asc\"\r\n",
                           "Content-Disposition: inline; filename=\"encrypted.asc\"\r\n",
                           "Content-Transfer-Encoding: 7bit\r\n",
                           "\r\n",
                           "-----BEGIN PGP MESSAGE-----\r\n",
                           "wV4D0dz5vDXklO8SAQdA5lGX1UU/eVQqDxNYdHa7tukoingHzqUB6wQssbMfHl8w\r\n",
                           "...\r\n",
                           "-----END PGP MESSAGE-----\r\n",
                           "\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK--\r\n"));
    }
    #[test]
    fn multi_part_signed() {
        let part = MultiPart::signed(
            "application/pgp-signature".to_owned(),
            "pgp-sha256".to_owned(),
        )
        .boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
        .part(Part::Single(
            SinglePart::builder()
                .header(header::ContentType("text/plain".parse().unwrap()))
                .body(String::from("Test email for signature")),
        ))
        .singlepart(
            SinglePart::builder()
                .header(ContentType(
                    "application/pgp-signature; name=\"signature.asc\""
                        .parse()
                        .unwrap(),
                ))
                .header(header::ContentDisposition {
                    disposition: header::DispositionType::Attachment,
                    parameters: vec![header::DispositionParam::Filename(
                        header::Charset::Ext("utf-8".into()),
                        None,
                        "signature.asc".into(),
                    )],
                })
                .body(String::from(concat!(
                    "-----BEGIN PGP SIGNATURE-----\r\n",
                    "\r\n",
                    "iHUEARYIAB0WIQTNsp3S/GbdE0KoiQ+IGQOscREZuQUCXyOzDAAKCRCIGQOscREZ\r\n",
                    "udgDAQCv3FJ3QWW5bRaGZAa0Ug6vASFdkvDMKoRwcoFnHPthjQEAiQ8skkIyE2GE\r\n",
                    "PoLpAXiKpT+NU8S8+8dfvwutnb4dSwM=\r\n",
                    "=3FYZ\r\n",
                    "-----END PGP SIGNATURE-----\r\n",
                ))),
        );

        assert_eq!(String::from_utf8(part.formatted()).unwrap(),
                   concat!("Content-Type: multipart/signed;",
                           " boundary=\"F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\";",
                           " protocol=\"application/pgp-signature\";",
                           " micalg=\"pgp-sha256\"\r\n",
                           "\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/plain\r\n",
                           "Content-Transfer-Encoding: 7bit\r\n",
                           "\r\n",
                           "Test email for signature\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: application/pgp-signature; name=\"signature.asc\"\r\n",
                           "Content-Disposition: attachment; filename=\"signature.asc\"\r\n",
                           "Content-Transfer-Encoding: 7bit\r\n",
                           "\r\n",
                           "-----BEGIN PGP SIGNATURE-----\r\n",
                            "\r\n",
                            "iHUEARYIAB0WIQTNsp3S/GbdE0KoiQ+IGQOscREZuQUCXyOzDAAKCRCIGQOscREZ\r\n",
                            "udgDAQCv3FJ3QWW5bRaGZAa0Ug6vASFdkvDMKoRwcoFnHPthjQEAiQ8skkIyE2GE\r\n",
                            "PoLpAXiKpT+NU8S8+8dfvwutnb4dSwM=\r\n",
                            "=3FYZ\r\n",
                            "-----END PGP SIGNATURE-----\r\n",
                           "\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK--\r\n"));
    }

    #[test]
    fn multi_part_alternative() {
        let part = MultiPart::alternative()
            .boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
            .part(Part::Single(SinglePart::builder()
                             .header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                             .header(header::ContentTransferEncoding::Binary)
                             .body(String::from("Текст письма в уникоде"))))
            .singlepart(SinglePart::builder()
                             .header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
                             .header(header::ContentTransferEncoding::Binary)
                             .body(String::from("<p>Текст <em>письма</em> в <a href=\"https://ru.wikipedia.org/wiki/Юникод\">уникоде</a><p>")));

        assert_eq!(String::from_utf8(part.formatted()).unwrap(),
                   concat!("Content-Type: multipart/alternative;",
                           " boundary=\"F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\"\r\n",
                           "\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "Текст письма в уникоде\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/html; charset=utf8\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "<p>Текст <em>письма</em> в <a href=\"https://ru.wikipedia.org/wiki/Юникод\">уникоде</a><p>\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK--\r\n"));
    }

    #[test]
    fn multi_part_mixed_related() {
        let part = MultiPart::mixed()
            .boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
            .multipart(MultiPart::related()
                            .boundary("E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh")
                            .singlepart(SinglePart::builder()
                                             .header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
                                             .header(header::ContentTransferEncoding::Binary)
                                             .body(String::from("<p>Текст <em>письма</em> в <a href=\"https://ru.wikipedia.org/wiki/Юникод\">уникоде</a><p>")))
                            .singlepart(SinglePart::builder()
                                             .header(header::ContentType("image/png".parse().unwrap()))
                                             .header(header::ContentLocation("/image.png".into()))
                                             .header(header::ContentTransferEncoding::Base64)
                                             .body(String::from("1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890"))))
            .singlepart(SinglePart::builder()
                             .header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                             .header(header::ContentDisposition {
                                 disposition: header::DispositionType::Attachment,
                                 parameters: vec![header::DispositionParam::Filename(header::Charset::Ext("utf-8".into()), None, "example.c".into())]
                             })
                             .header(header::ContentTransferEncoding::Binary)
                             .body(String::from("int main() { return 0; }")));

        assert_eq!(String::from_utf8(part.formatted()).unwrap(),
                   concat!("Content-Type: multipart/mixed;",
                           " boundary=\"F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\"\r\n",
                           "\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: multipart/related;",
                           " boundary=\"E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh\"\r\n",
                           "\r\n",
                           "--E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh\r\n",
                           "Content-Type: text/html; charset=utf8\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "<p>Текст <em>письма</em> в <a href=\"https://ru.wikipedia.org/wiki/Юникод\">уникоде</a><p>\r\n",
                           "--E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh\r\n",
                           "Content-Type: image/png\r\n",
                           "Content-Location: /image.png\r\n",
                           "Content-Transfer-Encoding: base64\r\n",
                           "\r\n",
                           "MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3\r\n",
                           "ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0\r\n",
                           "NTY3ODkwMTIzNDU2Nzg5MA==\r\n",
                           "--E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh--\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Disposition: attachment; filename=\"example.c\"\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "int main() { return 0; }\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK--\r\n"));
    }

    #[test]
    fn test_make_boundary() {
        let mut boundaries = std::collections::HashSet::with_capacity(10);
        for _ in 0..1000 {
            boundaries.insert(make_boundary());
        }

        // Ensure there are no duplicates
        assert_eq!(1000, boundaries.len());

        // Ensure correct length
        for boundary in boundaries {
            assert_eq!(40, boundary.len());
        }
    }
}
