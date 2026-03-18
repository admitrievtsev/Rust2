use crate::ast::{Constant, Pattern};

#[derive(Debug, Clone, PartialEq)]
pub enum LlExpr {
    Constant(Constant),
    Identifier(String),
    IfThenElse {
        condition: Box<LlExpr>,
        then_branch: Box<LlExpr>,
        else_branch: Box<LlExpr>,
    },
    Application {
        function: Box<LlExpr>,
        argument: Box<LlExpr>,
    },
    LetIn {
        is_rec: bool,
        name: Pattern,
        body: Box<LlExpr>,
        in_body: Box<LlExpr>,
    },
    Tuple(Vec<LlExpr>),
    Match {
        expr: Box<LlExpr>,
        branches: Vec<(Pattern, LlExpr)>,
    }, //TODO PM
}

#[derive(Debug, Clone, PartialEq)]
pub enum LlBinding {
    Let {
        name: Pattern,
        args: Vec<Pattern>,
        body: LlExpr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LlLetDeclaration {
    DSingleLet { is_rec: bool, body: LlBinding },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlProgram(pub Vec<LlLetDeclaration>);