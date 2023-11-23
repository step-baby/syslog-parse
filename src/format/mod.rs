use nom::bytes::complete::take_while1;
use nom::character::complete::digit1;
use nom::combinator::{map, map_res};
use nom::IResult;
use std::str::FromStr;

pub mod appname;
pub mod hostname;
pub mod msgid;
pub mod pri;
pub mod procid;
pub mod structured_data;
pub mod tagname;
pub mod timestamp;
pub mod version;

pub(crate) fn digits<T>(input: &str) -> IResult<&str, T>
where
    T: FromStr,
{
    map_res(digit1, FromStr::from_str)(input)
}

fn optional(input: &str, has_colons: bool) -> IResult<&str, Option<&str>> {
    map(
        // Note we need to use the ':' as a separator between the 3164 headers and the message.
        // So the header fields can't use them. Need to be aware of this to check
        // if this will be an issue.
        take_while1(|c: char| !c.is_whitespace() && (has_colons || c != ':')),
        |value: &str| {
            if value.is_empty() {
                None
            } else {
                Some(value)
            }
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let val = r#"1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - BOM'su root' failed for lonvick on /dev/pts/8"#;
        let d = digit1::<&str, nom::error::Error<&str>>(val).unwrap();
        println!("{:?}", d);

        let d1 = map(digit1::<&str, nom::error::Error<&str>>, |v| v)(val).unwrap();
        println!("{:?}", d1);
    }
}
