use crate::error::{New, NomError, NomErrorKind};
use crate::format::digits;
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::sequence::delimited;
use nom::IResult;

// 0~191
pub fn decompose_pri(pri: u8) -> Option<u8> {
    let f = (pri >> 3) as i32;
    let s = (pri & 0x7) as i32;

    if f.lt(&0) || f.gt(&23) {
        return None;
    }

    if s.lt(&0) || s.gt(&7) {
        return None;
    }

    Some(pri)
}

pub fn pri(input: &str) -> IResult<&str, String> {
    match opt(delimited(tag("<"), map(digits, decompose_pri), tag(">")))(input) {
        Ok((res, v)) => match v {
            Some(Some(v)) => Ok((res, format!("<{}>", v))),
            _ => Err(NomError::new(input, NomErrorKind::Fail)),
        },
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pri() {
        assert_eq!(pri("<165>").unwrap(), ("", "<165>".to_string())); // 20 * 8 + 5
        assert_eq!(pri("<193>").is_err(), true); // 24 * 8 + 1
        assert_eq!(pri("<190>").unwrap(), ("", "<190>".to_string()));
        assert_eq!(pri("<0>").unwrap(), ("", "<0>".to_string()));
    }
}
