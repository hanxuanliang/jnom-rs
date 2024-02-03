use crate::{IResult, Input};

#[derive(Debug)]
pub struct JError(pub String);

impl JError {
    pub fn from<O>(msg: &str) -> IResult<O> {
        Err(nom::Err::Error(JError(msg.to_string())))
    }
}

impl nom::error::ParseError<Input<'_>> for JError {
    fn from_error_kind(input: Input, kind: nom::error::ErrorKind) -> Self {
        JError(format!("Error: {:?} at {:?}", kind, input))
    }

    fn append(_: Input, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}
