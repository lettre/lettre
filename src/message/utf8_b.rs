// https://tools.ietf.org/html/rfc1522

fn allowed_char(c: char) -> bool {
    c >= 1 as char && c <= 9 as char
        || c == 11 as char
        || c == 12 as char
        || c >= 14 as char && c <= 127 as char
}

pub fn encode(s: &str) -> String {
    if s.chars().all(allowed_char) {
        s.into()
    } else {
        format!("=?utf-8?b?{}?=", base64::encode(s))
    }
}

pub fn decode(s: &str) -> Option<String> {
    s.strip_prefix("=?utf-8?b?")
        .and_then(|stripped| stripped.strip_suffix("?="))
        .map_or_else(
            || Some(s.into()),
            |stripped| {
                let decoded = base64::decode(stripped).ok()?;
                let decoded = String::from_utf8(decoded).ok()?;
                Some(decoded)
            },
        )
}

#[cfg(test)]
mod test {
    use super::{decode, encode};

    #[test]
    fn encode_ascii() {
        assert_eq!(&encode("Kayo. ?"), "Kayo. ?");
    }

    #[test]
    fn decode_ascii() {
        assert_eq!(decode("Kayo. ?"), Some("Kayo. ?".into()));
    }

    #[test]
    fn encode_utf8() {
        assert_eq!(
            &encode("Привет, мир!"),
            "=?utf-8?b?0J/RgNC40LLQtdGCLCDQvNC40YAh?="
        );
    }

    #[test]
    fn decode_utf8() {
        assert_eq!(
            decode("=?utf-8?b?0J/RgNC40LLQtdGCLCDQvNC40YAh?="),
            Some("Привет, мир!".into())
        );
    }
}
