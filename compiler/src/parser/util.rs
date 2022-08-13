use nom::{
    branch::alt,
    bytes::complete::tag,
    error::VerboseError,
    IResult,
};

pub fn endline(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    alt((tag("\r\n"), tag("\n")))(input)
}
