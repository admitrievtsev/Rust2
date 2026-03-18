use std::fmt;

#[derive(Debug, Clone, PartialEq)]
#[allow(unused)]
pub enum Token {
    // Ключевые слова
    Let,
    In,
    Fun,
    If,
    Then,
    Else,
    Match,
    With,
    Rec,
    And,

    // Литералы
    Int(i64),
    Bool(bool),
    String(String),

    // Операторы и разделители
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Arrow,
    Equal,
    Unit,
    Operator(String),

    // Идентификаторы
    Ident(String),

    // Конструкторы
    Constructor(String),

    // EOF
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Let => write!(f, "let"),
            Token::In => write!(f, "in"),
            Token::Fun => write!(f, "fun"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::Match => write!(f, "match"),
            Token::With => write!(f, "with"),
            Token::Rec => write!(f, "rec"),
            Token::And => write!(f, "and"),
            Token::Int(n) => write!(f, "{}", n),
            Token::Bool(b) => write!(f, "{}", b),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Arrow => write!(f, "->"),
            Token::Equal => write!(f, "="),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Constructor(s) => write!(f, "{}", s),
            Token::Eof => write!(f, "EOF"),
            Token::String(s) => write!(f, "{}", s),
            Token::Operator(op) => write!(f, "( {} )", op),
            Token::Unit => write!(f, "()")
        }
    }
}