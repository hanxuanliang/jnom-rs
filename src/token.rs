use std::ops::Range;

use logos::{Lexer, Logos};

pub struct JsonToken<'a> {
    pub source: &'a str,
    pub kind: JsonTokenKind,
    pub at: &'a str,
    pub span: Range<usize>,
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
