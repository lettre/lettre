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
    /// File name
    Attached(String),
    /// Content id
    Inline {
        content_id: String,
        name: Option<String>,
    },
}

impl Attachment {
    /// Create a new attachment
    ///
    /// This attachment will be displayed as a normal attachment,
    /// with the chosen `filename` appearing as the file name.
    ///
    /// ```rust
    /// # use std::error::Error;
    /// use std::fs;
    ///
    /// use lettre::message::{header::ContentType, Attachment};
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let filename = String::from("invoice.pdf");
    /// # if false {
    /// let filebody = fs::read("invoice.pdf")?;
    /// # }
    /// # let filebody = fs::read("docs/lettre.png")?;
    /// let content_type = ContentType::parse("application/pdf").unwrap();
    /// let attachment = Attachment::new(filename).body(filebody, content_type);
    ///
    /// // The document `attachment` will show up as a normal attachment.
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(filename: String) -> Self {
        Attachment {
            disposition: Disposition::Attached(filename),
        }
    }

    /// Create a new inline attachment
    ///
    /// This attachment should be displayed inline into the message
    /// body:
    ///
    /// ```html
    /// <img src="cid:123">
    /// ```
    ///
    ///
    /// ```rust
    /// # use std::error::Error;
    /// use std::fs;
    ///
    /// use lettre::message::{header::ContentType, Attachment};
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let content_id = String::from("123");
    /// # if false {
    /// let filebody = fs::read("image.jpg")?;
    /// # }
    /// # let filebody = fs::read("docs/lettre.png")?;
    /// let content_type = ContentType::parse("image/jpeg").unwrap();
    /// let attachment = Attachment::new_inline(content_id).body(filebody, content_type);
    ///
    /// // The image `attachment` will display inline into the email.
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_inline(content_id: String) -> Self {
        Attachment {
            disposition: Disposition::Inline {
                content_id,
                name: None,
            },
        }
    }

    /// Create a new inline attachment giving it a name
    ///
    /// This attachment should be displayed inline into the message
    /// body:
    ///
    /// ```html
    /// <img src="cid:123">
    /// ```
    ///
    ///
    /// ```rust
    /// # use std::error::Error;
    /// use std::fs;
    ///
    /// use lettre::message::{header::ContentType, Attachment};
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let content_id = String::from("123");
    /// let file_name = String::from("image.jpg");
    /// # if false {
    /// let filebody = fs::read(&file_name)?;
    /// # }
    /// # let filebody = fs::read("docs/lettre.png")?;
    /// let content_type = ContentType::parse("image/jpeg").unwrap();
    /// let attachment =
    ///     Attachment::new_inline_with_name(content_id, file_name).body(filebody, content_type);
    ///
    /// // The image `attachment` will display inline into the email.
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_inline_with_name(content_id: String, name: String) -> Self {
        Attachment {
            disposition: Disposition::Inline {
                content_id,
                name: Some(name),
            },
        }
    }

    /// Build the attachment into a [`SinglePart`] which can then be used to build the rest of the email
    ///
    /// Look at the [Complex MIME body example](crate::message#complex-mime-body)
    /// to see how [`SinglePart`] can be put into the email.
    pub fn body<T: IntoBody>(self, content: T, content_type: ContentType) -> SinglePart {
        let mut builder = SinglePart::builder();
        builder = match self.disposition {
            Disposition::Attached(filename) => {
                builder.header(header::ContentDisposition::attachment(&filename))
            }
            Disposition::Inline {
                content_id,
                name: None,
            } => builder
                .header(header::ContentId::from(format!("<{content_id}>")))
                .header(header::ContentDisposition::inline()),
            Disposition::Inline {
                content_id,
                name: Some(name),
            } => builder
                .header(header::ContentId::from(format!("<{content_id}>")))
                .header(header::ContentDisposition::inline_with_name(&name)),
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

    #[test]
    fn attachment_inline_with_name() {
        let id = String::from("id");
        let name = String::from("test");
        let part = super::Attachment::new_inline_with_name(id, name).body(
            String::from("Hello world!"),
            ContentType::parse("text/plain").unwrap(),
        );
        assert_eq!(
            &String::from_utf8_lossy(&part.formatted()),
            concat!(
                "Content-ID: <id>\r\n",
                "Content-Disposition: inline; filename=\"test\"\r\n",
                "Content-Type: text/plain\r\n",
                "Content-Transfer-Encoding: 7bit\r\n\r\n",
                "Hello world!\r\n"
            )
        );
    }
}
