//! Lexer for MiniML language

use std::iter::Peekable;
use std::str::Chars;

pub mod token;

pub use crate::lexer::token::Token;

#[derive(Clone)]
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    position: usize,
    prev_token: Option<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            position: 0,
            prev_token: None,
        }
    }

    fn is_oper_char(it: &char) -> bool {
        matches!(it, '$' | '&' | '*' | '+' | '-' | '/' | '=' | '>' | '@' | '^'| '~' | '!' | '?' | ':' | '.' | '<' | '%')
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

    pub fn next_token(&mut self) -> Token {
        // Пропускаем пробельные символы
        self.skip_whitespace();

        let token = match self.input.peek() {
            None => Token::Eof,
            Some(&ch) => {
                match ch {
                    '(' => {
                        self.bump();
                        self.skip_whitespace();
                        match self.input.peek() {
                            Some(&ch2) => {
                                match ch2 {
                                    ')' => {
                                        self.bump();
                                        Token::Unit
                                    }
                                    _ => {
                                        let mut operator = String::new();
                                        loop {
                                            self.skip_whitespace();
                                            let peeked_char = *self.input.peek().unwrap();
                                            if peeked_char == ')' {
                                                self.bump();
                                                return Token::Operator(operator);
                                            };
                                            if Self::is_oper_char(&peeked_char) {
                                                operator.push(peeked_char);
                                                self.bump();
                                            } else {
                                                return Token::LParen;
                                            }
                                        }
                                    }
                                }
                            }
                            _ => Token::LParen
                        }
                    }
                    ')' => {
                        self.bump();
                        Token::RParen
                    }
                    '{' => {
                        self.bump();
                        Token::LBrace
                    }
                    '}' => {
                        self.bump();
                        Token::RBrace
                    }
                    '[' => {
                        self.bump();
                        Token::LBracket
                    }
                    ']' => {
                        self.bump();
                        Token::RBracket
                    }
                    ',' => {
                        self.bump();
                        Token::Comma
                    }
                    ';' => {
                        self.bump();
                        Token::Semicolon
                    }
                    ':' => {
                        self.bump();
                        Token::Colon
                    }
                    '=' => {
                        self.bump();
                        match self.input.peek() {
                            Some(&'=') => {
                                self.bump();
                                Token::Operator("==".to_string())
                            }
                            _ => Token::Equal
                        }
                    }
                    '|' => {
                        self.bump();
                        Token::With
                    }
                    '>' => {
                        self.bump();
                        match self.input.peek() {
                            Some(&'=') => {
                                self.bump();
                                Token::Operator(">=".to_string())
                            }
                            _ => Token::Operator(">".to_string())
                        }
                    }
                    '<' => {
                        self.bump();
                        match self.input.peek() {
                            Some(&'=') => {
                                self.bump();
                                Token::Operator("<=".to_string())
                            }
                            _ => Token::Operator("<".to_string())
                        }
                    }
                    '!' => {
                        self.bump();
                        match self.input.peek() {
                            Some(&'=') => {
                                self.bump();
                                Token::Operator("!=".to_string())
                            }
                            _ => Token::Operator("!".to_string())
                        }
                    }
                    '-' => {
                        self.bump();
                        match self.input.peek() {
                            Some(&'>') => {
                                self.bump();
                                Token::Arrow
                            }
                            _ => Token::Operator("-".to_string())
                        }
                    }
                    '"' => {
                        self.bump();
                        self.read_string()
                    }
                    '+' => {
                        self.bump();
                        Token::Operator("+".to_string())
                    }
                    '*' => {
                        self.bump();
                        Token::Operator("*".to_string())
                    }
                    '/' => {
                        self.bump();
                        Token::Operator("/".to_string())
                    }
                    ch if Self::is_oper_char(&ch) => {
                        self.bump();
                        Token::Operator(ch.to_string())
                    }
                    ch if ch.is_ascii_digit() => {
                        self.read_number()
                    }
                    ch if ch.is_alphabetic() || ch == '_' => {
                        self.read_identifier()
                    }
                    _ => {
                        self.bump();
                        Token::Eof
                    }
                }
            }
        };
        self.prev_token = Some(token.clone());
        token
    }

    pub fn peek_token(&self) -> Option<Token> {
        // Create a temporary clone to peek without consuming
        let mut temp_lexer = self.clone();
        match temp_lexer.next_token() {
            Token::Eof => None,
            other => Some(other)
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.peek() {
            if ch.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn bump(&mut self) -> Option<char> {
        self.position += 1;
        self.input.next()
    }

    fn read_string(&mut self) -> Token {
        let mut str = String::new();

        while let Some(&ch) = self.input.peek() {
            if ch != '"' {
                str.push(ch);
                self.bump();
            } else {
                self.bump();
                break;
            }
        }

        Token::String(str)
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();

        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.bump();
            } else {
                break;
            }
        }

        Token::Int(num_str.parse::<i64>().unwrap_or(0))
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident_str = String::new();

        while let Some(&ch) = self.input.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident_str.push(ch);
                self.bump();
            } else {
                break;
            }
        }

        // Check if it's a keyword
        match ident_str.as_str() {
            "let" => Token::Let,
            "in" => Token::In,
            "fun" => Token::Fun,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "match" => Token::Match,
            "with" => Token::With,
            "rec" => Token::Rec,
            "and" => Token::And,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            _ => Token::Ident(ident_str),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("let x = 42");
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Int(42));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_unary_tokens() {
        let mut lexer = Lexer::new("let x = -42");
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Ident("-".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(42));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("fun if then else");
        assert_eq!(lexer.next_token(), Token::Fun);
        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::Then);
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("( ) , ; : -> =");
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::Colon);
        assert_eq!(lexer.next_token(), Token::Arrow);
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_boolean_literals() {
        let mut lexer = Lexer::new("true false");
        assert_eq!(lexer.next_token(), Token::Bool(true));
        assert_eq!(lexer.next_token(), Token::Bool(false));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("x y_var identifier123");
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("y_var".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("identifier123".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("let rec a = \"123\" let b = 1 let c = true");
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Rec);
        assert_eq!(lexer.next_token(), Token::Ident("a".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::String("123".to_string()));
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident("b".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Int(1));
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident("c".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Bool(true));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 123 0");
        assert_eq!(lexer.next_token(), Token::Int(42));
        assert_eq!(lexer.next_token(), Token::Int(123));
        assert_eq!(lexer.next_token(), Token::Int(0));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("  \t\n  let  \t\n  x  \t\n  =  \t\n  42  \t\n  ");
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Int(42));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_complex_expressions() {
        let mut lexer = Lexer::new("if (x + y) then true else false");
        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("+".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("y".to_string()));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::Then);
        assert_eq!(lexer.next_token(), Token::Bool(true));
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::Bool(false));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_single_character_tokens() {
        let mut lexer = Lexer::new("()[]{}");
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::LBracket);
        assert_eq!(lexer.next_token(), Token::RBracket);
        assert_eq!(lexer.next_token(), Token::LBrace);
        assert_eq!(lexer.next_token(), Token::RBrace);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_function_definition() {
        let mut lexer = Lexer::new("fun x -> x + 1");
        assert_eq!(lexer.next_token(), Token::Fun);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Arrow);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("+".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(1));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_tuple_expression() {
        let mut lexer = Lexer::new("(1, 2, 3)");
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Int(1));
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Int(2));
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Int(3));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_list_expression() {
        let mut lexer = Lexer::new("[1; 2; 3]");
        assert_eq!(lexer.next_token(), Token::LBracket);
        assert_eq!(lexer.next_token(), Token::Int(1));
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::Int(2));
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::Int(3));
        assert_eq!(lexer.next_token(), Token::RBracket);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_nested_expressions() {
        let mut lexer = Lexer::new("let x = (1 + 2) * 3 in x");
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Int(1));
        assert_eq!(lexer.next_token(), Token::Ident("+".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(2));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::Ident("*".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(3));
        assert_eq!(lexer.next_token(), Token::In);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_recursive_function() {
        let mut lexer = Lexer::new("let rec fact = fun n -> if n <= 1 then 1 else n * fact (n - 1) in fact 5");
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Rec);
        assert_eq!(lexer.next_token(), Token::Ident("fact".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Fun);
        assert_eq!(lexer.next_token(), Token::Ident("n".to_string()));
        assert_eq!(lexer.next_token(), Token::Arrow);
        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::Ident("n".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("<=".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(1));
        assert_eq!(lexer.next_token(), Token::Then);
        assert_eq!(lexer.next_token(), Token::Int(1));
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::Ident("n".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("*".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("fact".to_string()));
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Ident("n".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("-".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(1));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::In);
        assert_eq!(lexer.next_token(), Token::Ident("fact".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(5));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_comparison_operators() {
        let mut lexer = Lexer::new(">= <= != > < ==");
        assert_eq!(lexer.next_token(), Token::Ident(">=".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("<=".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("!=".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident(">".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("<".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_complex_let_expression_with_recursion() {
        let mut lexer = Lexer::new("let rec fib = fun n -> if n <= 1 then n else fib (n - 1) + fib (n - 2) in fib 10");
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Rec);
        assert_eq!(lexer.next_token(), Token::Ident("fib".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Fun);
        assert_eq!(lexer.next_token(), Token::Ident("n".to_string()));
        assert_eq!(lexer.next_token(), Token::Arrow);
        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::Ident("n".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("<=".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(1));
        assert_eq!(lexer.next_token(), Token::Then);
        assert_eq!(lexer.next_token(), Token::Ident("n".to_string()));
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::Ident("fib".to_string()));
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Ident("n".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("-".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(1));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::Ident("+".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("fib".to_string()));
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Ident("n".to_string()));
        assert_eq!(lexer.next_token(), Token::Ident("-".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(2));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::In);
        assert_eq!(lexer.next_token(), Token::Ident("fib".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(10));
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}