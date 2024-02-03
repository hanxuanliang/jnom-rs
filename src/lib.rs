#![allow(dead_code)]

use error::JError;
use indexmap::IndexMap;
use nom::{
    branch::alt,
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, tuple},
    Slice,
};
use token::{JsonToken, JsonTokenKind};

mod common;
mod error;
mod token;

pub type Input<'a> = &'a [JsonToken<'a>];
pub type IResult<'a, Output> = nom::IResult<Input<'a>, Output, error::JError>;

#[derive(Debug, PartialEq)]
enum JsonExpr<'a> {
    Object(Box<IndexMap<&'a str, JsonExpr<'a>>>),
    Array(Vec<JsonExpr<'a>>),
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}

fn parse_json(i: Input) -> IResult<JsonExpr> {
    alt((
        parse_obj,
        parse_array,
        parse_string,
        parse_number,
        parse_bool,
    ))(i)
}

fn parse_obj(i: Input) -> IResult<JsonExpr> {
    delimited(
        match_token(JsonTokenKind::OpenBrace),
        separated_list0(
            match_token(JsonTokenKind::Comma),
            tuple((parse_string, match_token(JsonTokenKind::Colon), parse_json)),
        ),
        match_token(JsonTokenKind::CloseBrace),
    )(i)
    .map(|(i, map_var)| {
        let out = map_var
            .into_iter()
            .map(|(k, _, v)| match k {
                JsonExpr::String(k) => (k, v),
                _ => unreachable!(),
            })
            .collect::<IndexMap<_, _>>();

        (i, JsonExpr::Object(Box::new(out)))
    })
}

fn parse_array(i: Input) -> IResult<JsonExpr> {
    tuple((
        match_token(JsonTokenKind::OpenBracket),
        separated_list0(match_token(JsonTokenKind::Comma), parse_json),
        match_token(JsonTokenKind::CloseBracket),
    ))(i)
    .map(|(i, (_, array_var, _))| (i, JsonExpr::Array(array_var)))
}

fn parse_string(i: Input) -> IResult<JsonExpr> {
    match i.get(0) {
        Some(JsonToken {
            kind: JsonTokenKind::String(s),
            ..
        }) => Ok((i.slice(1..), JsonExpr::String(s.trim_matches('"')))),
        _ => Err(nom::Err::Error(JError(format!(
            "JsonToken Kind String does not match"
        )))),
    }
}

fn parse_number(i: Input) -> IResult<JsonExpr> {
    match i.get(0) {
        Some(JsonToken {
            kind: JsonTokenKind::Number(n),
            ..
        }) => Ok((i.slice(1..), JsonExpr::Number(*n))),
        _ => Err(nom::Err::Error(JError(format!(
            "JsonToken Kind Number does not match"
        )))),
    }
}

fn parse_bool(i: Input) -> IResult<JsonExpr> {
    alt((
        map(match_token(JsonTokenKind::True), |_| {
            JsonExpr::Boolean(true)
        }),
        map(match_token(JsonTokenKind::False), |_| {
            JsonExpr::Boolean(false)
        }),
    ))(i)
}

fn match_token(kind: JsonTokenKind) -> impl Fn(Input) -> IResult<&JsonToken> {
    move |i| match i.get(0).filter(|token| token.kind == kind) {
        Some(token) => Ok((i.slice(1..), token)),
        None => Err(nom::Err::Error(JError(format!(
            "JsonToken Kind {kind} does not match",
        )))),
    }
}

fn match_text(text: &'static str) -> impl Fn(Input) -> IResult<&JsonToken> {
    move |i| match i.get(0).filter(|token| token.text() == text) {
        Some(token) => Ok((i.slice(1..), token)),
        None => Err(nom::Err::Error(JError(format!(
            "Json Text {text} does not match",
        )))),
    }
}

#[cfg(test)]
mod tests {
    use crate::token::tokenize;
    use crate::JsonExpr;

    #[test]
    fn it_tokenize() {
        let source = r#"
            {
                "name": "John Doe",
                "age": 30,
                "isStudent": false,
                "scores": [100, 90, 95],
                "address": {
                    "street": "123 Main St",
                    "city": "Springfield",
                    "state": "IL"
                }
            }
        "#;
        let tokens = tokenize(source);
        println!("{:#?}", tokens);
    }

    #[test]
    fn it_parse_string() {
        let source = "\"abc\"";
        let tokens = tokenize(source);
        let result = super::parse_string(&tokens);
        let string_var = result.unwrap().1;
        assert_eq!(string_var, JsonExpr::String("abc"));
    }

    #[test]
    fn it_parse_number() {
        let source = "123";
        let tokens = tokenize(source);
        let result = super::parse_number(&tokens);
        let number_var = result.unwrap().1;

        assert_eq!(number_var, JsonExpr::Number(123.0));
    }

    #[test]
    fn it_parse_bool() {
        let source = "true";
        let tokens = tokenize(source);
        let result = super::parse_bool(&tokens);
        let bool_var = result.unwrap().1;

        assert_eq!(bool_var, JsonExpr::Boolean(true));
    }

    #[test]
    fn it_parse_array() {
        let source = r#"["abc", "def"]"#;
        let tokens = tokenize(source);
        let result = super::parse_array(&tokens);
        let array_var = result.unwrap().1;

        assert_eq!(
            array_var,
            JsonExpr::Array(vec![JsonExpr::String("abc"), JsonExpr::String("def"),])
        );
    }

    #[test]
    fn it_parse_obj() {
        let source = r#"{"name": "John Doe", "address": "杭州"}"#;
        let tokens = tokenize(source);
        let result = super::parse_obj(&tokens);
        let obj_var = result.unwrap().1;

        assert_eq!(
            obj_var,
            JsonExpr::Object(Box::new(
                vec![
                    ("name", JsonExpr::String("John Doe")),
                    ("address", JsonExpr::String("杭州")),
                ]
                .into_iter()
                .collect()
            ))
        );
    }

    #[test]
    fn it_parse_nest_obj() {
        let source = r#"
            {
                "name": "John Doe", 
                "address": {"city": "Springfield", "state": [1, 12]}
            }"#;
        let tokens = tokenize(source);
        let result = super::parse_obj(&tokens);
        let obj_var = result.unwrap().1;

        assert_eq!(
            obj_var,
            JsonExpr::Object(Box::new(
                vec![
                    ("name", JsonExpr::String("John Doe")),
                    (
                        "address",
                        JsonExpr::Object(Box::new(
                            vec![
                                ("city", JsonExpr::String("Springfield")),
                                (
                                    "state",
                                    JsonExpr::Array(vec![
                                        JsonExpr::Number(1.0),
                                        JsonExpr::Number(12.0)
                                    ])
                                ),
                            ]
                            .into_iter()
                            .collect()
                        ))
                    ),
                ]
                .into_iter()
                .collect()
            ))
        );
    }
}
