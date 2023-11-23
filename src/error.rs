use nom::error::Error;

pub type ParseResult<'a, T> = Result<T, SyslogParseError<'a>>;
pub type NomErrorKind = nom::error::ErrorKind;
pub type NomError<'a> = nom::Err<Error<&'a str>>;

#[derive(Error, Debug)]
pub enum SyslogParseError<'a> {
    #[error("parse {0}: {1}")]
    ParseError0(&'a str, nom::Err<Error<&'a str>>),

    #[error("parse {0}: {1}")]
    ParseError1(&'a str, ErrorKind),

    #[error("时间格式不符合，期待1985-04-12T23:20:50.52Z、1985-04-12T19:20:50.52-04:00、2003-10-11T22:14:15.003Z等")]
    TimeParseError,
}

#[derive(Debug, Display)]
pub enum ErrorKind {
    #[strum(serialize = "优先级格式不符合，希望facility范围:0-23，severity范围：0-7")]
    Pri,
    #[strum(
        serialize = "时间格式不符合，期待1985-04-12T23:20:50.52Z、1985-04-12T19:20:50.52-04:00、2003-10-11T22:14:15.003Z等"
    )]
    Timestamp,
    #[strum(serialize = "Hostname格式不符合, IPV4遵循RFC1035，IPV6遵循RFC4291")]
    Hostname,
    #[strum(serialize = "Appname格式不符合")]
    Appname,
    #[strum(serialize = "Process Id格式不符合")]
    ProcessId,
    #[strum(serialize = "Message Id格式不符合")]
    MessageId,
}

pub trait New {
    #[allow(clippy::new_ret_no_self)]
    fn new(input: &str, code: nom::error::ErrorKind) -> NomError {
        nom::Err::Error(Error::new(input, code))
    }
}

impl New for NomError<'_> {}
