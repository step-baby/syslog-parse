use crate::error::{ParseResult, SyslogParseError};
use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_until};
use nom::character::complete::{digit1, space1};
use nom::combinator::{map, map_res, opt};
use nom::sequence::tuple;
use nom::IResult;

fn parse_timestamp(input: &str) -> ParseResult<&str> {
    if chrono::DateTime::parse_from_rfc3339(input).is_ok() {
        Ok(input)
    } else {
        Err(SyslogParseError::TimeParseError)
    }
}

pub(crate) fn timestamp_3339(input: &str) -> IResult<&str, String> {
    match map_res(take_until(" "), parse_timestamp)(input) {
        Ok((res, t)) => Ok((res, t.to_string())),
        Err(e) => Err(e),
    }
}

/// The month as a three letter string. Returns the number.
fn parse_month(s: &str) -> Result<&str, String> {
    match s.to_lowercase().as_ref() {
        "jan" | "feb" | "mar" | "apr" | "may" | "jun" | "jul" | "aug" | "sep" | "oct" | "nov"
        | "dec" => Ok(s),
        _ => Err(format!("Invalid month {}", s)),
    }
}

/// The timestamp for 3164 messages. MMM DD HH:MM:SS
fn timestamp_3164_no_year(input: &str) -> IResult<&str, String> {
    map(
        tuple((
            map_res(take(3_usize), parse_month),
            space1,
            digit1,
            space1,
            digit1,
            tag(":"),
            digit1,
            tag(":"),
            digit1,
            opt(tag(":")),
        )),
        |(month, s0, date, s1, hour, _, minute, _, seconds, s4)| {
            let mut value = format!(
                "{}{}{}{}{}:{}:{}",
                month, s0, date, s1, hour, minute, seconds
            );
            if s4.is_some() {
                value = format!("{}:", value);
            }
            value
        },
    )(input)
}

/// Timestamp including year. MMM DD YYYY HH:MM:SS
fn timestamp_3164_with_year(input: &str) -> IResult<&str, String> {
    map(
        tuple((
            map_res(take(3_usize), parse_month),
            space1,
            digit1,
            space1,
            digit1,
            space1,
            digit1,
            tag(":"),
            digit1,
            tag(":"),
            digit1,
            opt(tag(":")),
        )),
        |(month, s0, date, s1, year, s2, hour, _, minute, _, seconds, s5)| {
            let mut value = format!(
                "{}{}{}{}{}{}{}:{}:{}",
                month, s0, date, s1, year, s2, hour, minute, seconds
            );
            if s5.is_some() {
                value = format!("{}:", value);
            }
            value
        },
    )(input)
}

/// Parse the timestamp in the format specified in RFC3164,
/// either with year or without.
/// MMM DD HH:MM:SS or MMM DD YYYY HH:MM:SS
//
/// # Arguments
///
/// * get_year - a function that is called if the parsed message contains a date with no year.
///              the function takes a (month, date, hour, minute, second) tuple and should return the year to use.
/// * tz - An optional timezone.
///        If None is specified and the parsed date doesn't specify a timezone the date is parsed in time local time.
///
pub(crate) fn timestamp_3164(input: &str) -> IResult<&str, String> {
    alt((
        map(timestamp_3164_no_year, |ts| ts),
        map(timestamp_3164_with_year, |naive_date| naive_date),
        timestamp_3339,
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        let resp = timestamp_3339("1985-04-12T23:20:50.52Z ").unwrap();
        assert_eq!(resp, (" ", "1985-04-12T23:20:50.52Z".to_string()));

        let resp = timestamp_3339("1985-04-12T19:20:50.52-04:00 ").unwrap();
        assert_eq!(resp, (" ", "1985-04-12T19:20:50.52-04:00".to_string()));

        let resp = timestamp_3339("2003-10-11T22:14:15.003Z ").unwrap();
        assert_eq!(resp, (" ", "2003-10-11T22:14:15.003Z".to_string()));

        let resp = timestamp_3339("2003-08-24T05:14:15.000003-07:00 ").unwrap();
        assert_eq!(resp, (" ", "2003-08-24T05:14:15.000003-07:00".to_string()));

        let resp = timestamp_3339("2003-08-24T05:14:15.000000003-07:00 ").unwrap();
        assert_eq!(
            resp,
            (" ", "2003-08-24T05:14:15.000000003-07:00".to_string())
        );
    }

    #[test]
    fn test_timestamp_3164() {
        let t = r#"JAN 15 20:00:32 "#;
        let resp = timestamp_3164(t).unwrap();
        assert_eq!(resp, (" ", "JAN 15 20:00:32".to_string()));

        let t = r#"JAN 15 2023 20:00:32 "#;
        let resp = timestamp_3164(t).unwrap();
        assert_eq!(resp, (" ", "JAN 15 2023 20:00:32".to_string()));
    }
}
