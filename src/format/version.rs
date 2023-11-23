use nom::character::complete::digit1;
use nom::IResult;

pub fn version(input: &str) -> IResult<&str, &str> {
    digit1(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version("1").unwrap(), ("", "1"));
    }
}
