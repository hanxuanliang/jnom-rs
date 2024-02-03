use crate::{
    error::JError,
    token::{JsonToken, JsonTokenKind},
    IResult, Input,
};
use nom::Slice;

pub fn match_token(kind: JsonTokenKind) -> impl Fn(Input) -> IResult<&JsonToken> {
    move |i| match i.get(0).filter(|token| token.kind == kind) {
        Some(token) => Ok((i.slice(1..), token)),
        None => Err(nom::Err::Error(JError(format!(
            "JsonToken Kind {kind} does not match",
        )))),
    }
}

pub fn match_text(text: &'static str) -> impl Fn(Input) -> IResult<&JsonToken> {
    move |i| match i.get(0).filter(|token| token.text() == text) {
        Some(token) => Ok((i.slice(1..), token)),
        None => Err(nom::Err::Error(JError(format!(
            "Json Text {text} does not match",
        )))),
    }
}
