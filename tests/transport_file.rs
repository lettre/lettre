#[cfg(all(feature = "file-transport", feature = "builder"))]
fn default_date() -> std::time::SystemTime {
    use std::time::{Duration, SystemTime};

    // Tue, 15 Nov 1994 08:12:31 GMT
    SystemTime::UNIX_EPOCH + Duration::from_secs(784887151)
}

#[cfg(test)]
#[cfg(all(feature = "file-transport", feature = "builder"))]
mod sync {
    use std::{
        env::temp_dir,
        fs::{read_to_string, remove_file},
    };

    use lettre::{FileTransport, Message, Transport};

    use crate::default_date;

    #[test]
    fn file_transport() {
        let sender = FileTransport::new(temp_dir());
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date(default_date())
            .body(String::from("Be happy!"))
            .unwrap();

        let result = sender.send(&email);
        let id = result.unwrap();

        let eml_file = temp_dir().join(format!("{}.eml", id));
        let eml = read_to_string(&eml_file).unwrap();

        assert_eq!(
            eml,
            concat!(
                "From: NoBody <nobody@domain.tld>\r\n",
                "Reply-To: Yuin <yuin@domain.tld>\r\n",
                "To: Hei <hei@domain.tld>\r\n",
                "Subject: Happy new year\r\n",
                "Date: Tue, 15 Nov 1994 08:12:31 -0000\r\n",
                "Content-Transfer-Encoding: 7bit\r\n",
                "\r\n",
                "Be happy!"
            )
        );
        remove_file(eml_file).unwrap();
    }

    #[test]
    #[cfg(feature = "file-transport-envelope")]
    fn file_transport_with_envelope() {
        let sender = FileTransport::with_envelope(temp_dir());
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date(default_date())
            .body(String::from("Be happy!"))
            .unwrap();

        let result = sender.send(&email);
        let id = result.unwrap();

        let eml_file = temp_dir().join(format!("{}.eml", id));
        let eml = read_to_string(&eml_file).unwrap();

        let json_file = temp_dir().join(format!("{}.json", id));
        let json = read_to_string(&json_file).unwrap();

        assert_eq!(
            eml,
            concat!(
                "From: NoBody <nobody@domain.tld>\r\n",
                "Reply-To: Yuin <yuin@domain.tld>\r\n",
                "To: Hei <hei@domain.tld>\r\n",
                "Subject: Happy new year\r\n",
                "Date: Tue, 15 Nov 1994 08:12:31 -0000\r\n",
                "Content-Transfer-Encoding: 7bit\r\n",
                "\r\n",
                "Be happy!"
            )
        );

        assert_eq!(
            json,
            "{\"forward_path\":[\"hei@domain.tld\"],\"reverse_path\":\"nobody@domain.tld\"}"
        );

        let (e, m) = sender.read(&id).unwrap();

        assert_eq!(&e, email.envelope());
        assert_eq!(m, email.formatted());

        remove_file(eml_file).unwrap();
        remove_file(json_file).unwrap();
    }
}

#[cfg(test)]
#[cfg(all(feature = "file-transport", feature = "builder", feature = "tokio1"))]
mod tokio_1 {
    use std::{
        env::temp_dir,
        fs::{read_to_string, remove_file},
    };

    use lettre::{AsyncFileTransport, AsyncTransport, Message, Tokio1Executor};
    use tokio1_crate as tokio;

    use crate::default_date;

    #[tokio::test]
    async fn file_transport_tokio1() {
        let sender = AsyncFileTransport::<Tokio1Executor>::new(temp_dir());
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date(default_date())
            .body(String::from("Be happy!"))
            .unwrap();

        let result = sender.send(email).await;
        let id = result.unwrap();

        let eml_file = temp_dir().join(format!("{}.eml", id));
        let eml = read_to_string(&eml_file).unwrap();

        assert_eq!(
            eml,
            concat!(
                "From: NoBody <nobody@domain.tld>\r\n",
                "Reply-To: Yuin <yuin@domain.tld>\r\n",
                "To: Hei <hei@domain.tld>\r\n",
                "Subject: Happy new year\r\n",
                "Date: Tue, 15 Nov 1994 08:12:31 -0000\r\n",
                "Content-Transfer-Encoding: 7bit\r\n",
                "\r\n",
                "Be happy!"
            )
        );
        remove_file(eml_file).unwrap();
    }
}

#[cfg(test)]
#[cfg(all(
    feature = "file-transport",
    feature = "builder",
    feature = "async-std1"
))]
mod asyncstd_1 {
    use std::{
        env::temp_dir,
        fs::{read_to_string, remove_file},
    };

    use lettre::{AsyncFileTransport, AsyncStd1Executor, AsyncTransport, Message};

    use crate::default_date;

    #[async_std::test]
    async fn file_transport_asyncstd1() {
        let sender = AsyncFileTransport::<AsyncStd1Executor>::new(temp_dir());
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date(default_date())
            .body(String::from("Be happy!"))
            .unwrap();

        let result = sender.send(email).await;
        let id = result.unwrap();

        let eml_file = temp_dir().join(format!("{}.eml", id));
        let eml = read_to_string(&eml_file).unwrap();

        assert_eq!(
            eml,
            concat!(
                "From: NoBody <nobody@domain.tld>\r\n",
                "Reply-To: Yuin <yuin@domain.tld>\r\n",
                "To: Hei <hei@domain.tld>\r\n",
                "Subject: Happy new year\r\n",
                "Date: Tue, 15 Nov 1994 08:12:31 -0000\r\n",
                "Content-Transfer-Encoding: 7bit\r\n",
                "\r\n",
                "Be happy!"
            )
        );
        remove_file(eml_file).unwrap();
    }
}
