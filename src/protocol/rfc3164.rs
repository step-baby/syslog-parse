use crate::format::hostname::hostname;
use crate::format::pri::pri;
use crate::format::structured_data::structured_data_optional;
use crate::format::tagname::tagname;
use crate::format::timestamp::timestamp_3164;
use crate::protocol::Msg;
use nom::bytes::complete::tag;
use nom::character::complete::space0;
use nom::combinator::{map, opt, rest};
use nom::sequence::{preceded, tuple};
use nom::IResult;

pub(crate) fn parse(input: &str) -> IResult<&str, Msg> {
    map(
        tuple((
            pri,
            opt(space0),
            timestamp_3164,
            opt(preceded(tag(" "), hostname)),
            opt(preceded(tag(" "), tagname)),
            opt(space0),
            opt(tag(":")),
            opt(space0),
            opt(structured_data_optional(false)),
            opt(space0),
            rest,
        )),
        |(pri, s1, timestamp, field1, field2, s2, s3, s4, structured_data, s5, msg)| {
            let header = format!(
                "{}{}{}{}{}{}{}{}{}{}",
                pri,
                s1.unwrap_or_default(),
                timestamp,
                field1.unwrap_or_default().unwrap_or_default(),
                field2.unwrap_or_default().unwrap_or_default(),
                s2.unwrap_or_default(),
                s3.unwrap_or_default(),
                s4.unwrap_or_default(),
                structured_data.unwrap_or_default(),
                s5.unwrap_or_default(),
            );
            Msg { header, msg }
        },
    )(input)
}
