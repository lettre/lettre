#[cfg(test)]
#[cfg(all(feature = "file-transport", feature = "builder"))]
mod sync {
    use lettre::{FileTransport, Message, Transport};
    use std::{
        env::temp_dir,
        fs::{read_to_string, remove_file},
    };

    #[test]
    fn file_transport() {
        let sender = FileTransport::new(temp_dir());
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
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
                "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
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
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
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
                "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
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
#[cfg(all(feature = "file-transport", feature = "builder", feature = "tokio02"))]
mod tokio_02 {
    use lettre::{AsyncFileTransport, AsyncTransport, Message, Tokio02Executor};
    use std::{
        env::temp_dir,
        fs::{read_to_string, remove_file},
    };

    use tokio02_crate as tokio;

    #[tokio::test]
    async fn file_transport_tokio02() {
        let sender = AsyncFileTransport::<Tokio02Executor>::new(temp_dir());
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
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
                "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
                "Content-Transfer-Encoding: 7bit\r\n",
                "\r\n",
                "Be happy!"
            )
        );
        remove_file(eml_file).unwrap();
    }
}

#[cfg(test)]
#[cfg(all(feature = "file-transport", feature = "builder", feature = "tokio1"))]
mod tokio_1 {
    use lettre::{AsyncFileTransport, AsyncTransport, Message, Tokio1Executor};
    use std::{
        env::temp_dir,
        fs::{read_to_string, remove_file},
    };

    use tokio1_crate as tokio;

    #[cfg(feature = "tokio02")]
    #[tokio::test]
    async fn file_transport_tokio1() {
        let sender = AsyncFileTransport::<Tokio1Executor>::new(temp_dir());
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
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
                "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
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
    use lettre::{AsyncFileTransport, AsyncStd1Executor, AsyncTransport, Message};
    use std::{
        env::temp_dir,
        fs::{read_to_string, remove_file},
    };

    #[async_std::test]
    async fn file_transport_asyncstd1() {
        let sender = AsyncFileTransport::<AsyncStd1Executor>::new(temp_dir());
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .date("Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap())
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
                "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
                "Content-Transfer-Encoding: 7bit\r\n",
                "\r\n",
                "Be happy!"
            )
        );
        remove_file(eml_file).unwrap();
    }
}
