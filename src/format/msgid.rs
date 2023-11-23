use crate::format::optional;
use nom::IResult;

pub(crate) fn msgid(input: &str) -> IResult<&str, Option<&str>> {
    optional(input, false)
}
