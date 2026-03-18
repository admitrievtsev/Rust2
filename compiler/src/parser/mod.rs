use crate::ast::Expr::Function;
use crate::ast::Pattern::{Identifier, Oper};
use crate::ast::{Constant, Expr, LetDeclaration, Program, SingleLet};
use crate::lexer::{Lexer, Token};

pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    program: Program,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Parser {
            lexer: Lexer::new(input),
            program: Program { items: vec![] },
        }
    }

    pub(crate) fn parse_program(&mut self) -> Program {
        while self.lexer.peek_token().is_some() {
            let token = self.lexer.next_token();
            match token {
                Token::Let => {
                    let new_decl = self.parse_decl();

                    self.program.items.push(new_decl);
                }
                token => panic!("Unexpected token {} encountered during parsing.", token)
            }
        }
        self.program.clone()
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_let_in_expr()
    }

    fn parse_let_in_expr(&mut self) -> Expr {
        if let Some(Token::Let) = self.lexer.peek_token() {
            self.lexer.next_token();
            let mut is_rec = false;
            if let Some(Token::Rec) = self.lexer.peek_token() {
                self.lexer.next_token();
                is_rec = true
            }
            let name = Box::new(self.parse_pattern().unwrap());

            let body = self.parse_let_binding();

            match self.lexer.next_token() {
                Token::In => (),
                token => panic!("Expected 'in' keyword after let expression, found {} instead.", token),
            }

            let in_expr = Box::new(self.parse_expr());

            return Expr::LetIn { is_rec, name, body, in_expr };
        }

        self.parse_expr_precedence(0)
    }

    fn parse_pattern(&mut self) -> Option<crate::ast::Pattern> {
        match self.lexer.peek_token() {
            Some(Token::Ident(name)) => {
                self.lexer.next_token();
                Some(Identifier(name))
            }
            Some(Token::Unit) => {
                self.lexer.next_token();
                Some(crate::ast::Pattern::Constant(Constant::Unit))
            }
            _ => None,
        }
    }

    fn parse_expr_precedence(&mut self, min_precedence: u8) -> Expr {
        let mut expr = self.parse_primary_expr();

        loop {
            let next_token = self.lexer.peek_token();
            let current_precedence = match next_token {
                Some(Token::Operator(ref op)) if op == "+" || op == "-" => 10,
                Some(Token::Operator(ref op)) if op == "*" || op == "/" => 20,
                Some(Token::Operator(ref op)) if op == "<" || op == ">" || op == "<=" || op == ">=" || op == "==" || op == "<>" => 5,
                Some(Token::Operator(ref op)) if op == "&&" || op == "||" => 4,
                Some(Token::Operator(_)) => 1,
                Some(Token::Equal) => 0,
                _ => break,
            };

            if current_precedence < min_precedence {
                break;
            }

            let op = self.lexer.next_token();

            let rhs = self.parse_expr_precedence(current_precedence + 1);

            expr = match op {
                Token::Operator(ref op) if op == "+" || op == "-" || op == "*" || op == "/" ||
                    op == "<" || op == ">" || op == "<=" || op == ">=" ||
                    op == "==" || op == "<>" || op == "&&" || op == "||" => {
                    Expr::Application(
                        Box::new(Expr::Application(
                            Box::new(Expr::Identifier(format!("({})", op.clone()))),
                            Box::new(expr),
                        )),
                        Box::new(rhs),
                    )
                }
                Token::Equal => {
                    Expr::Application(
                        Box::new(Expr::Application(
                            Box::new(Expr::Identifier(format!("({})", op.clone()))),
                            Box::new(expr),
                        )),
                        Box::new(rhs),
                    )
                }
                Token::Operator(op) => {
                    Expr::Application(
                        Box::new(Expr::Application(
                            Box::new(Expr::Identifier(format!("({})", op.clone()))),
                            Box::new(expr),
                        )),
                        Box::new(rhs),
                    )
                }
                _ => {
                    expr
                }
            };
        }

        expr
    }

    /// Parse a comma-separated list of expressions
    fn parse_comma_separated_exprs(&mut self) -> Vec<Expr> {
        let mut exprs = vec![];

        loop {
            let expr = self.parse_expr();
            exprs.push(expr);

            match self.lexer.peek_token() {
                Some(Token::Comma) => {
                    self.lexer.next_token(); // consume the comma
                    continue;
                }
                _ => break,
            }
        }

        exprs
    }

    /// Parse primary expressions with proper left-associative application handling.
    fn parse_primary_expr(&mut self) -> Expr {
        let mut expr = self.parse_atom_expr().unwrap();

        while let Some(Token::Fun | Token::If | Token::Int(_) | Token::Bool(_) | Token::Unit | Token::Ident(_) | Token::LParen | Token::String(_)) = self.lexer.peek_token() {
            let next_expr = self.parse_atom_expr();
            if let Some(atom) = next_expr {
                expr = Expr::Application(Box::new(expr), Box::new(atom));
            } else {
                break;
            }
        }
        expr
    }

    /// Parse atomic expressions (base expressions that don't involve application).
    fn parse_atom_expr(&mut self) -> Option<Expr> {
        let token = self.lexer.peek_token()?;
        match token {
            Token::Fun | Token::If | Token::Int(_) | Token::Bool(_) | Token::Unit | Token::Ident(_) | Token::LParen | Token::String(_) => {
                match self.lexer.next_token() {
                    Token::If => {
                        let condition = Box::new(self.parse_expr());
                        self.lexer.next_token(); // consume 'then'
                        let then_branch = Box::new(self.parse_expr());
                        let mut else_branch = None;
                        if let Some(Token::Else) = self.lexer.peek_token() {
                            self.lexer.next_token();
                            else_branch = Some(Box::new(self.parse_expr()));
                        }
                        Some(Expr::IfThenElse { condition, then_branch, else_branch })
                    }
                    Token::Int(val) => {
                        match self.parse_atom_expr() {
                            Some(expr) => Some(Expr::Application(
                                Box::new(Expr::Constant(Constant::Int(val))),
                                Box::new(expr),
                            )),
                            None => Some(Expr::Constant(Constant::Int(val))),
                        }
                    }
                    Token::Bool(val) => Some(Expr::Constant(Constant::Bool(val))),
                    Token::Unit => Some(Expr::Constant(Constant::Unit)),
                    Token::Ident(name) => {
                        Some(Expr::Identifier(name))
                    }
                    Token::Fun => {
                        let name = self.parse_pattern()?;

                        let body = self.parse_fun_binding();
                        Some(Expr::Function(name, body))
                    }
                    Token::LParen => {
                        let peeked_token = self.lexer.peek_token();
                        match peeked_token {
                            Some(Token::RParen) => {
                                self.lexer.next_token();
                                match self.parse_atom_expr() {
                                    Some(new_expr) => Some(Expr::Application(
                                        Box::new(Expr::Constant(Constant::Unit)),
                                        Box::new(new_expr),
                                    )),
                                    None => Some(Expr::Constant(Constant::Unit)),
                                }
                            }
                            _ => {
                                let exprs = self.parse_comma_separated_exprs();

                                match self.lexer.next_token() {
                                    Token::RParen => {
                                        let expr = if exprs.len() > 1 {
                                            Expr::Tuple(exprs)
                                        } else if exprs.len() == 1 {
                                            exprs.first().cloned().unwrap()
                                        } else {
                                            Expr::Constant(Constant::Unit)
                                        };
                                        Some(expr)
                                    }
                                    _ => panic!("Expected closing parenthesis"),
                                }
                            }
                        }
                    }
                    Token::String(_) => {
                        Some(Expr::Constant(crate::ast::Constant::Int(0))) // TODO Placeholder
                    }
                    Token::Eof => None,
                    _ => None,
                }
            }
            _ => None
        }
    }

    fn parse_let_binding(&mut self) -> Box<Expr> {
        match self.parse_pattern() {
            Some(pt) => {
                Box::from(Function(pt, self.parse_let_binding()))
            }
            None => match self.lexer.peek_token() {
                Some(Token::Equal) => {
                    self.lexer.next_token();
                    let expr = self.parse_expr();
                    Box::from(expr)
                }
                _ => panic!("Unexpected token after 'let' in {}.", self.lexer.get_position())
            }
        }
    }

    fn parse_fun_binding(&mut self) -> Box<Expr> {
        match self.parse_pattern() {
            Some(pt) => {
                Box::from(Function(pt, self.parse_fun_binding()))
            }
            None => match self.lexer.peek_token() {
                Some(Token::Arrow) => {
                    self.lexer.next_token();
                    let expr = self.parse_expr();
                    Box::from(expr)
                }
                _ => panic!("Unexpected token after 'let' in {}.", self.lexer.get_position())
            }
        }
    }

    fn parse_single_let(&mut self) -> SingleLet {
        let name = match self.lexer.next_token() {
            Token::Ident(name) => Identifier(name),
            Token::Operator(op) => Oper(op),
            _ => panic!("Expected an identifier after 'let' in {}.", self.lexer.get_position()),
        };

        let binding = self.parse_let_binding();


        SingleLet {
            name: Box::new(name),
            binding,
        }
    }

    fn parse_decl(&mut self) -> LetDeclaration {
        match self.lexer.peek_token() {
            Some(Token::Rec) => {
                self.lexer.next_token();
                LetDeclaration::SingleLet { is_rec: true, single_let: self.parse_single_let() }
            }
            Some(Token::Ident(_)) => {
                LetDeclaration::SingleLet { is_rec: false, single_let: self.parse_single_let() }
            }
            Some(Token::Operator(_)) => {
                LetDeclaration::SingleLet { is_rec: false, single_let: self.parse_single_let() }
            }
            _ => panic!("Expected an identifier or rec after 'let' in {}.", self.lexer.get_position()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;


    #[test]
    fn test_parse_simple_int() {
        let input = "let main = let x = 42 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_infix_operator() {
        let input = "let ( $ ) a b = a + b let f = 1 $ b";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:#?}", program);
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_simple_bool() {
        let input = "let main = let x = true in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_simple_identifier() {
        let input = "let main = let x = y in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_nested_parentheses() {
        let input = "let main = let x = (1 + 2) in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_simple_addition() {
        let input = "let main = let x = 1 + 2 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_multiplication_precedence() {
        let input = "let main = let x = 2 * 3 + 1 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_complex_expression() {
        let input = "let main = let x = (1 + 2) * 3 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_if_then_else() {
        let input = "let main = let x = if true then 1 else 2 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_multiple_declarations() {
        let input = "let main = let x = 1 in let y = 2 in x + y";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_function_application() {
        let input = "let main = let x = f 1 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_recursion() {
        let input = "let main = let rec fact = fun n -> if n <= 1 then 1 else n * fact (n - 1) in fact 5";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_tuple_expression() {
        let input = "let main = let x = (1, 2, 3) in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_complex_nested_expressions() {
        let input = "let main = let x = ((1 + 2) * 3) + 4 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_comparison_operators() {
        let input = "let main = let x = 1 < 2 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_logical_operators() {
        let input = "let main = let x = true in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_string_literal() {
        let input = r#"let main = let x = "hello" in x"#;
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_unit_expression() {
        let input = "let main = let x = () in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_complex_arithmetic() {
        let input = "let main = let x = 10 - 5 + 3 * 2 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_nested_if_expressions() {
        let input = "let main = let x = if true then (if false then 1 else 2) else 3 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_function_with_multiple_args() {
        let input = "let main = let x = fun a b -> a + b in x 1 2";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:#?}", program);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_complex_nested_parentheses() {
        let input = "let main = let x = (((1 + 2) * 3) + 4) in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_mixed_operations() {
        let input = "let main =let x = 1 + 2 * 3 - 4 / 2 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_complex_conditionals() {
        let input = "let main = let x = if (1 + 2) > (3 - 1) then true else false in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:#?}", program);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_expression_with_variables() {
        let input = "let main = let x = 1 in let y = 2 in x + y";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_nested_function_calls() {
        let input = "let main = let x = f (g  2 1) in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_let_in_expression() {
        let input = "let main = let x = 1 in let y = 2 in x + y";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_complex_nested_expressions_with_functions() {
        let input = "let main = let x = fun a -> a + 1 in let y = x 5 in y * 2";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:?}", program);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_expression_with_precedence() {
        let input = "let main = let x = 2 + 3 * 4 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_complex_arithmetic_with_parentheses() {
        let input = "let main = let x = (2 + 3) * 4 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_nested_conditionals() {
        let input = "let main = let x = if true then if false then 1 else 2 else 3 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_expression_with_function_application() {
        let input = "let main = let x = f 1 2 3 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:?}", program);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_empty_program() {
        let input = "";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 0);
    }

    #[test]
    fn test_parse_only_whitespace() {
        let input = "   \t\n  ";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 0);
    }

    #[test]
    fn test_parse_complex_expression_chain() {
        let input = "let main = let x = 1 + 2 * 3 - 4 / 2 + 5 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_mixed_expressions() {
        let input = "let main = let x = (1 + 2) * (3 - 4) in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_tupled_expressions() {
        let input = "let main = let x = (1, 2) + (3, 4) + (5, 6, \"asd\") in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:?}", program);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_nested_function_application() {
        let input = "let f = 5 let y u = u * 5 let p = print_int ((y 6) + 1)";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:?}", program.items);
        assert_eq!(program.items.len(), 3);
    }

    #[test]
    fn test_parse_complex_function_with_conditions() {
        let input = "let main = let x = fun n -> if n > 0 then n else 0 in x 5";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_multiple_expressions_in_sequence() {
        let input = "let main = let x = 1 in let y = 2 in let z = 3 in x + y + z";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_expression_with_all_operators() {
        let input = "let main = let x = 1 + 2 * 3 - 4 / 5 < 6 > 7 <= 8 >= 9 = 10 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        assert_eq!(program.items.len(), 1);
    }
    #[test]
    fn test_fac() {
        let input = "let rec fac n = if n <= 1 then 1 else n * fac (n - 1) let main = let () = print_int (fac 4) in 0";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:?}", program);
        assert_eq!(program.items.len(), 2);
    }
}