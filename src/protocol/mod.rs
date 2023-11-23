use nom::branch::alt;
use nom::IResult;

mod rfc3164;
mod rfc5424;

#[derive(Debug, Eq, PartialEq)]
pub struct Msg<'a> {
    pub header: String,
    pub msg: &'a str,
}

pub fn single_parse(input: &str) -> IResult<&str, Msg> {
    alt((rfc5424::parse, rfc3164::parse))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let msg =  "<11>1 2023-09-07T09:45:08.899092Z localhost <90>myprogram5424 42 1545121 - tcp 传输 syslog";
        let value = single_parse(msg).unwrap();
        let exptecd_msg = "tcp 传输 syslog";
        assert_eq!(value, ("", Msg{
            header: "<11>1 2023-09-07T09:45:08.899092Z localhost <90>myprogram5424 42 1545121 - ".to_string(),
            msg: exptecd_msg,
        }));
    }
}
