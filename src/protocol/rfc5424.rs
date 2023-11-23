use crate::format::appname::appname;
use crate::format::hostname::hostname;
use crate::format::msgid::msgid;
use crate::format::pri::pri;
use crate::format::procid::procid;
use crate::format::structured_data::structured_data;
use crate::format::timestamp::timestamp_3339;
use crate::format::version::version;
use crate::protocol::Msg;
use nom::character::complete::{space0, space1};
use nom::combinator::{map, rest};
use nom::sequence::tuple;
use nom::IResult;

/// Parse the message as per RFC5424
pub(crate) fn parse(input: &str) -> IResult<&str, Msg> {
    match map(
        tuple((
            pri,
            version,
            space1,
            timestamp_3339,
            space1,
            hostname,
            space1,
            appname,
            space1,
            procid,
            space1,
            msgid,
            space0,
            structured_data,
            space0,
            rest,
        )),
        |(
            pri,
            version,
            s1,
            timestamp,
            s2,
            hostname,
            s3,
            appname,
            s4,
            procid,
            s5,
            msgid,
            s6,
            structured_data,
            s7,
            rest_msg,
        )| {
            let header = format!(
                "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                pri,
                version,
                s1,
                timestamp,
                s2,
                hostname.unwrap_or_default(),
                s3,
                appname.unwrap_or_default(),
                s4,
                procid.unwrap_or_default(),
                s5,
                msgid.unwrap_or_default(),
                s6,
                structured_data,
                s7
            );
            Msg {
                header,
                msg: rest_msg,
            }
        },
    )(input)
    {
        Ok(value) => Ok(value),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let msg = "<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - BOM'su root' failed for lonvick on /dev/pts/8";
        let value = parse(msg).unwrap();
        let expected_msg = "BOM'su root' failed for lonvick on /dev/pts/8";
        assert_eq!(
            value,
            (
                "",
                Msg {
                    header: "<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - "
                        .to_string(),
                    msg: expected_msg,
                }
            )
        );
    }

    #[test]
    fn test_2() {
        let msg = "<165>1 2003-08-24T05:14:15.000003-07:00 192.0.2.1 myproc 8710 - - %% It's time to make the do-nuts";
        let value = parse(msg).unwrap();
        let expected_msg = "%% It's time to make the do-nuts";
        assert_eq!(
            value,
            (
                "",
                Msg {
                    header: "<165>1 2003-08-24T05:14:15.000003-07:00 192.0.2.1 myproc 8710 - - "
                        .to_string(),
                    msg: expected_msg,
                }
            )
        );
    }

    #[test]
    fn test_3() {
        let msg = r#"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"] BOMAn application event log entry"#;
        let value = parse(msg).unwrap();
        let expected_msg = "BOMAn application event log entry";
        assert_eq!(value, ("", Msg{
            header: r#"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"] "#.to_string(),
            msg: expected_msg,
        }));
    }

    #[test]
    fn test_4() {
        let msg = r#"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"][examplePriority@32473 class="high"]"#;
        let value = parse(msg).unwrap();
        let expected_msg = "";
        assert_eq!(value, ("", Msg{
            header: r#"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"][examplePriority@32473 class="high"]"#.to_string(),
            msg: expected_msg,
        }));
    }
}
