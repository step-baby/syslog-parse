use crate::format::optional;
use nom::IResult;

pub(crate) fn hostname(input: &str) -> IResult<&str, Option<&str>> {
    optional(input, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hostname() {
        assert_eq!(
            hostname("198.128.24.221 ").unwrap(),
            (" ", Some("198.128.24.221"))
        );
    }
}
