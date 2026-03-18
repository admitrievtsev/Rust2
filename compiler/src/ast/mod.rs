//! Abstract Syntax Tree for Rust2 language

#[derive(Debug, Clone, PartialEq)]
#[derive(Hash)]
#[derive(Eq)]
#[allow(unused)]
pub enum Constant {
    Int(i64),
    Bool(bool),
    Nil, // unimplemented
    Unit,
}
#[derive(Debug, Clone)]
#[derive(Eq, Hash, PartialEq)]
#[allow(unused)]
pub enum Pattern {
    WildCard, // TODO PM
    Cons(Box<Pattern>, Box<Pattern>), // TODO lists
    Identifier(String),
    Tuple(Vec<Pattern>),
    Constant(Constant),
    Oper(String),
}

/// Used for LLVM codegen typing
/// #[allow(unused)]
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[allow(unused)]
pub enum TypeName {
    Unit,
    Int,
    Bool,
    Poly(String),
    Function(Box<TypeName>, Box<TypeName>),
    List(Box<TypeName>), // TODO
}

#[derive(Debug, Clone, PartialEq)]
#[allow(unused)]
pub enum Expr {
    Constant(Constant),
    Identifier(String),
    Function(Pattern, Box<Expr>),
    Application(Box<Expr>, Box<Expr>),
    IfThenElse { condition: Box<Expr>, then_branch: Box<Expr>, else_branch: Option<Box<Expr>> },
    LetIn {
        is_rec: bool,
        name: Box<Pattern>,
        body: Box<Expr>,
        in_expr: Box<Expr>,
    },
    Tuple(Vec<Expr>),
    Match(Box<Expr>, Vec<(Pattern, Expr)>),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SingleLet {
    pub(crate) name: Box<Pattern>,
    pub(crate) binding: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LetDeclaration {
    SingleLet
    {
        is_rec: bool,
        single_let: SingleLet,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<LetDeclaration>,
}
