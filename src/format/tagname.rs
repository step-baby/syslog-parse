use crate::format::optional;
use nom::IResult;

pub(crate) fn tagname(input: &str) -> IResult<&str, Option<&str>> {
    optional(input, false)
}
