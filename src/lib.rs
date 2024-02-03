#![allow(dead_code)]
use std::ops::Range;

use error::JError;
use indexmap::IndexMap;
use logos::{Lexer, Logos};
use nom::{
    multi::separated_list0,
    sequence::{delimited, tuple},
    Slice,
};

mod error;

pub struct JsonLexer<'a> {
    source: &'a str,
    lexer: Lexer<'a, JsonTokenKind>,
}

pub struct JsonToken<'a> {
    source: &'a str,
    pub kind: JsonTokenKind,
    at: &'a str,
    span: Range<usize>,
}

impl<'a> JsonToken<'a> {
    pub fn text(&self) -> &'a str {
        &self.source[self.span.clone()]
    }
}

impl std::fmt::Debug for JsonToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}[{:?}] @ {}..{}",
            self.kind,
            self.text(),
            self.span.start,
            self.span.end
        )
    }
}

impl<'a> JsonLexer<'a> {
    pub fn new(source: &'a str) -> Self {
        JsonLexer {
            source,
            lexer: JsonTokenKind::lexer(source),
        }
    }
}

impl<'a> Iterator for JsonLexer<'a> {
    type Item = JsonToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.next() {
            Some(Ok(token)) => Some(JsonToken {
                source: self.source,
                kind: token,
                at: self.lexer.slice(),
                span: self.lexer.span(),
            }),
            _ => None,
        }
    }
}

// tokenize Tokenize the input string
pub fn tokenize(source: &str) -> Vec<JsonToken> {
    JsonLexer::new(source).collect::<Vec<_>>()
}

#[derive(Logos, Debug, PartialEq)]
pub enum JsonTokenKind {
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,

    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,

    #[token(":")]
    Colon,
    #[token(",")]
    Comma,

    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,

    #[regex(r"-?\d+(\.\d+)?([eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap_or_default())]
    Number(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    String(String),

    #[regex(r"\s+", logos::skip)]
    Whitespace,
}

impl std::fmt::Display for JsonTokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonTokenKind::OpenBrace => write!(f, "{{"),
            JsonTokenKind::CloseBrace => write!(f, "}}"),
            JsonTokenKind::OpenBracket => write!(f, "["),
            JsonTokenKind::CloseBracket => write!(f, "]"),
            JsonTokenKind::Colon => write!(f, ":"),
            JsonTokenKind::Comma => write!(f, ","),
            JsonTokenKind::True => write!(f, "true"),
            JsonTokenKind::False => write!(f, "false"),
            JsonTokenKind::Null => write!(f, "null"),
            JsonTokenKind::Number(n) => write!(f, "{}", n),
            JsonTokenKind::String(s) => write!(f, "{}", s),
            JsonTokenKind::Whitespace => write!(f, " "),
        }
    }
}

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

fn parse_obj(i: Input) -> IResult<JsonExpr> {
    delimited(
        match_token(JsonTokenKind::OpenBrace),
        separated_list0(
            match_token(JsonTokenKind::Comma),
            tuple((
                parse_string,
                match_token(JsonTokenKind::Colon),
                parse_string,
            )),
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
        separated_list0(match_token(JsonTokenKind::Comma), parse_string),
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
    use crate::JsonExpr;

    #[test]
    // cargo test --package jnom-rs --lib -- tests::it_tokenize --exact --nocapture
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
        let tokens = super::tokenize(source);
        println!("{:#?}", tokens);
    }

    #[test]
    fn it_parse_string() {
        let source = "\"abc\"";
        let tokens = super::tokenize(source);
        // println!("{:#?}", tokens);
        let result = super::parse_string(&tokens);
        let string_var = result.unwrap().1;
        assert_eq!(string_var, JsonExpr::String("abc"));
    }

    #[test]
    fn it_parse_array() {
        let source = r#"["abc", "def"]"#;
        let tokens = super::tokenize(source);
        // println!("{:#?}", tokens);
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
        let tokens = super::tokenize(source);
        // println!("{:#?}", tokens);
        let result = super::parse_obj(&tokens);
        // println!("{:#?}", result);
        let obj_var = result.unwrap().1;
        println!("{:#?}", obj_var);
    }
}
