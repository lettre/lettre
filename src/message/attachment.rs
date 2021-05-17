use crate::message::{
    header::{self, ContentType},
    IntoBody, SinglePart,
};

/// `SinglePart` builder for attachments
///
/// Allows building attachment parts easily.
#[derive(Clone)]
pub struct Attachment {
    disposition: Disposition,
}

#[derive(Clone)]
enum Disposition {
    /// file name
    Attached(String),
    /// content id
    Inline(String),
}

impl Attachment {
    /// Creates a new attachment
    pub fn new(filename: String) -> Self {
        Attachment {
            disposition: Disposition::Attached(filename),
        }
    }

    /// Creates a new inline attachment
    pub fn new_inline(content_id: String) -> Self {
        Attachment {
            disposition: Disposition::Inline(content_id),
        }
    }

    /// Build the attachment part
    pub fn body<T: IntoBody>(self, content: T, content_type: ContentType) -> SinglePart {
        let mut builder = SinglePart::builder();
        builder = match self.disposition {
            Disposition::Attached(filename) => {
                builder.header(header::ContentDisposition::attachment(&filename))
            }
            Disposition::Inline(content_id) => builder
                .header(header::ContentId::from(format!("<{}>", content_id)))
                .header(header::ContentDisposition::inline()),
        };
        builder = builder.header(content_type);
        builder.body(content)
    }
}

#[cfg(test)]
mod tests {
    use crate::message::header::ContentType;

    #[test]
    fn attachment() {
        let part = super::Attachment::new(String::from("test.txt")).body(
            String::from("Hello world!"),
            ContentType::parse("text/plain").unwrap(),
        );
        assert_eq!(
            &String::from_utf8_lossy(&part.formatted()),
            concat!(
                "Content-Disposition: attachment; filename=\"test.txt\"\r\n",
                "Content-Type: text/plain\r\n",
                "Content-Transfer-Encoding: 7bit\r\n\r\n",
                "Hello world!\r\n",
            )
        );
    }

    #[test]
    fn attachment_inline() {
        let part = super::Attachment::new_inline(String::from("id")).body(
            String::from("Hello world!"),
            ContentType::parse("text/plain").unwrap(),
        );
        assert_eq!(
            &String::from_utf8_lossy(&part.formatted()),
            concat!(
                "Content-ID: <id>\r\n",
                "Content-Disposition: inline\r\n",
                "Content-Type: text/plain\r\n",
                "Content-Transfer-Encoding: 7bit\r\n\r\n",
                "Hello world!\r\n"
            )
        );
    }
}
