use crate::format::optional;
use nom::IResult;

pub(crate) fn appname(input: &str) -> IResult<&str, Option<&str>> {
    optional(input, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appname() {
        let value = appname("syslog").unwrap();
        assert_eq!(value, ("", Some("syslog")));
    }
}
