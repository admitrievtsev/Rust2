//! ANF (A-normal form) types for MiniML language

#[derive(Debug, Clone, PartialEq)]
pub enum ImmExpr {
    Int(i64),
    Identifier(String),
    Bool(bool),
    Unit,
    Nil,
    Tuple(Vec<ImmExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CExpr {
    Application(Box<ImmExpr>, Box<ImmExpr>, Vec<ImmExpr>),
    IfThenElse { condition: Box<ImmExpr>, then_branch: AExpr, else_branch: AExpr },
    ImmExpr(Box<ImmExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AExpr {
    LetIn { name: String, body: Box<CExpr>, in_body: Box<AExpr> },
    CExpr(Box<CExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SingleAnfBinding {
    Let { name: String, args: Vec<String>, body: AExpr },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnfDecl {
    SingleLet
    {
        is_rec: bool,
        single_anf_binding: SingleAnfBinding,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnfProg(pub Vec<AnfDecl>);
