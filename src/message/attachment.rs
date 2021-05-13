use crate::message::{
    header::{self, Header},
    IntoBody, SinglePart,
};
use mime::Mime;

#[derive(Clone, Copy)]
enum Disposition {
    Attachment,
    Inline,
}

/// `SinglePart` builder for attachments
///
/// Allows building attachment parts easily.
pub struct Attachment {
    filename: Option<String>,
    content_disposition: Disposition,
    content_type: Option<Mime>,
    content_id: Option<String>,
}

impl Default for Attachment {
    fn default() -> Self {
        Self::new()
    }
}

impl Attachment {
    /// Creates a new attachment
    pub fn new() -> Self {
        Self {
            filename: None,
            content_disposition: Disposition::Attachment,
            content_type: None,
            content_id: None,
        }
    }

    /// Creates a new inline attachment
    pub fn new_inline() -> Self {
        Self {
            filename: None,
            content_disposition: Disposition::Inline,
            content_type: None,
            content_id: None,
        }
    }

    /// Specify a MIME type for the attachment
    pub fn content_type(mut self, content_type: Mime) -> Self {
        self.content_type = Some(content_type);
        self
    }

    /// Specify a file name (optional for inline attachments)
    pub fn filename(mut self, filename: String) -> Self {
        self.filename = Some(filename);
        self
    }

    /// For use in inline attachments to allow referencing it.
    ///
    /// The `<>` are inserted automatically.
    pub fn content_id(mut self, content_id: String) -> Self {
        self.content_id = Some(format!("<{}>", content_id));
        self
    }

    /// Build the attachment part
    ///
    /// This method panics if used with attachment disposition and no
    /// filename was provided.
    pub fn body<T: IntoBody>(self, content: T) -> SinglePart {
        let mut builder = SinglePart::builder();

        builder = match self.content_disposition {
            Disposition::Attachment => match self.filename {
                Some(filename) => builder.header(header::ContentDisposition::attachment(&filename)),
                None => panic!("Missing filename attachment"),
            },
            Disposition::Inline => match self.filename {
                Some(filename) => {
                    builder.header(header::ContentDisposition::inline_with_name(&filename))
                }
                None => builder.header(header::ContentDisposition::inline()),
            },
        };

        if let Some(content_type) = self.content_type {
            builder = builder.header(header::ContentType::from_mime(content_type))
        }

        if let Some(content_id) = self.content_id {
            builder = builder.header(header::ContentId::from(&content_id))
        }

        builder.body(content)
    }
}

mod tests {
    #[test]
    fn attachment() {
        let part = super::Attachment::new()
            .filename(String::from("test.txt"))
            .content_type("test/plain".parse().unwrap())
            .body(String::from("Hello world!"));
        assert_eq!(
            &String::from_utf8_lossy(&part.formatted()),
            concat!(
                "Content-Disposition: attachment; filename=\"test.txt\"\r\n",
                "Content-Type: test/plain\r\n",
                "Content-Transfer-Encoding: 7bit\r\n\r\n",
                "Hello world!\r\n",
            )
        );
    }

    #[test]
    fn attachment_inline() {
        let part = super::Attachment::new_inline()
            .content_id(String::from("id"))
            .content_type("test/plain".parse().unwrap())
            .body(String::from("Hello world!"));
        assert_eq!(
            &String::from_utf8_lossy(&part.formatted()),
            concat!(
                "Content-Disposition: inline\r\n",
                "Content-Type: test/plain\r\n",
                "Content-ID: <id>\r\n",
                "Content-Transfer-Encoding: 7bit\r\n\r\n",
                "Hello world!\r\n"
            )
        );
    }
}
