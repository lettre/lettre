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
    const PREFIX: &str = "=?utf-8?b?";
    const SUFFIX: &str = "?=";

    let s = s.trim();
    if s.starts_with(PREFIX) && s.ends_with(SUFFIX) {
        let s = &s[PREFIX.len()..];
        let s = &s[..s.len() - SUFFIX.len()];
        base64::decode(s)
            .ok()
            .and_then(|v| String::from_utf8(v).ok())
    } else {
        Some(s.into())
    }
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
