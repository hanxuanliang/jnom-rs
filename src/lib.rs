#![allow(dead_code)]
use logos::{Lexer, Logos};

pub struct JsonLexer<'a> {
    source: &'a str,
    lexer: Lexer<'a, JsonTokenKind>,
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
    type Item = JsonTokenKind;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.next() {
            Some(Ok(token)) => return Some(token),
            Some(Err(_)) => return None,
            None => return None,
        }
    }
}

// tokenize Tokenize the input string
pub fn tokenize(source: &str) -> Vec<JsonTokenKind> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
