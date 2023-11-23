use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_till1, take_until, take_while1},
    character::complete::{anychar, space0},
    combinator::map,
    multi::{many1, separated_list0},
    sequence::{delimited, separated_pair, terminated, tuple},
    IResult,
};

fn value0(input: &str) -> IResult<&str, String> {
    match map(tag(r#""""#), |_| "".to_string())(input) {
        Ok((res, v)) => Ok((res, v)),
        Err(e) => Err(e),
    }
}

fn value1(input: &str) -> IResult<&str, String> {
    match delimited(
        tag("\""),
        escaped(take_while1(|c: char| c != '\\' && c != '"'), '\\', anychar),
        tag("\""),
    )(input)
    {
        Ok((res, v)) => Ok((res, format!("\"{}\"", v))),
        Err(e) => Err(e),
    }
}

/// Parse the param value - a string delimited by '"' - '\' escapes \ and "
fn param_value(input: &str) -> IResult<&str, String> {
    match alt((value0, value1))(input) {
        Ok((res, v)) => Ok((res, v)),
        Err(e) => Err(e),
    }
}

/// Parse a param name="value"
fn param(input: &str) -> IResult<&str, String> {
    match separated_pair(
        take_till1(|c: char| c == ']' || c == '='),
        terminated(tag("="), space0),
        param_value,
    )(input)
    {
        Ok((res, (k, v))) => Ok((res, format!("{}={}", k, v))),
        Err(e) => Err(e),
    }
}

/// Parse a single structured data record.
/// [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"]
fn structured_datum_strict(input: &str) -> IResult<&str, String> {
    match delimited(
        tag("["),
        map(
            tuple((
                take_till1(|c: char| c.is_whitespace() || c == ']' || c == '='),
                space0,
                separated_list0(tag(" "), param),
            )),
            |(id, space, params)| format!("{}{}{}", id, space, params.join(" ")),
        ),
        tag("]"),
    )(input)
    {
        Ok((res, v)) => Ok((res, format!("[{}]", v))),
        Err(e) => Err(e),
    }
}

fn inner_permissive(input: &str) -> IResult<&str, String> {
    match delimited(tag("["), map(take_until("]"), |v: &str| v), tag("]"))(input) {
        Ok((res, v)) => Ok((res, format!("[{}]", v))),
        Err(e) => Err(e),
    }
}

// /// Parse a single structured data record allowing anything between brackets.
fn structured_datum_permissive(input: &str) -> IResult<&str, String> {
    alt((
        structured_datum_strict,
        // If the element fails to parse, just parse it and return None.
        inner_permissive,
    ))(input)
}
//
// /// Parse a single structured data record.
fn structured_datum(allow_failure: bool) -> impl FnMut(&str) -> IResult<&str, String> {
    if allow_failure {
        structured_datum_permissive
    } else {
        structured_datum_strict
    }
}

/// Parse multiple structured data elements.
pub(crate) fn structured_data_optional(
    allow_failure: bool,
) -> impl FnMut(&str) -> IResult<&str, String> {
    move |input| {
        alt((
            map(tag("-"), |_| "-".to_string()),
            map(many1(structured_datum(allow_failure)), |items| {
                items.join("")
            }),
        ))(input)
    }
}

/// Parse multiple structured data elements.
pub(crate) fn structured_data(input: &str) -> IResult<&str, String> {
    structured_data_optional(true)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_data() {
        let data = r#"[exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"]"#;
        let value = structured_data(data).unwrap();
        assert_eq!(data, value.1);

        let data = r#"[exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"][examplePriority@32473 class="high"]"#;
        let value = structured_data(data).unwrap();
        assert_eq!(data, value.1);
    }
}
