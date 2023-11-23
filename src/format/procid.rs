use crate::format::optional;
use nom::IResult;

pub(crate) fn procid(input: &str) -> IResult<&str, Option<&str>> {
    optional(input, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procid() {
        let val = "- ID47 - BOM'su root' failed for lonvick on /dev/pts/8";
        let resp = procid(val).unwrap();
        // let resp = take_while1::<_, &str, nom::error::Error<&str>>(|c: char| !c.is_whitespace() && (false || c != ':'))(val).unwrap();
        println!("{:?}", resp);
    }
}
