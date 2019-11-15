//! Utils for string manipulation

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Encode a string as xtext
#[derive(Debug)]
#[cfg_attr(
    feature = "serde-impls",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct XText<'a>(pub &'a str);

impl<'a> Display for XText<'a> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut rest = self.0;
        while let Some(idx) = rest.find(|c| c < '!' || c == '+' || c == '=') {
            let (start, end) = rest.split_at(idx);
            f.write_str(start)?;

            let mut end_iter = end.char_indices();
            let (_, c) = end_iter.next().expect("char");
            write!(f, "+{:X}", c as u8)?;

            if let Some((idx, _)) = end_iter.next() {
                rest = &end[idx..];
            } else {
                rest = "";
            }
        }
        f.write_str(rest)
    }
}

#[cfg(test)]
mod tests {
    use super::XText;

    #[test]
    fn test() {
        for (input, expect) in [
            ("bjorn", "bjorn"),
            ("bjørn", "bjørn"),
            ("Ø+= ❤️‰", "Ø+2B+3D+20❤️‰"),
            ("+", "+2B"),
        ]
        .iter()
        {
            assert_eq!(format!("{}", XText(input)), expect.to_string());
        }
    }
}
