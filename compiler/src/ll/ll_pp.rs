use crate::ast::{Constant, Pattern};
use crate::ll::ll_ast::{LlExpr, LlLetDeclaration, LlProgram};
use std::fmt::Write;

/// Format a constant value
pub fn frestore_constant<W: Write>(writer: &mut W, c: &Constant) -> core::fmt::Result<> {
    match c {
        Constant::Int(i) => write!(writer, "{}", i),
        Constant::Bool(false) => write!(writer, "false"),
        Constant::Bool(true) => write!(writer, "true"),
        Constant::Nil => write!(writer, "[]"),
        Constant::Unit => write!(writer, "()"),
    }
}

/// Format a pattern
pub fn frestore_pattern<W: Write>(writer: &mut W, pat: &Pattern) -> core::fmt::Result<> {
    match pat {
        Pattern::WildCard => write!(writer, "_"),
        Pattern::Cons(h_pat, t_pat) => {
            write!(writer, "(")?;
            frestore_pattern(writer, h_pat)?;
            write!(writer, " :: ")?;
            frestore_pattern(writer, t_pat)?;
            write!(writer, ")")
        }
        Pattern::Identifier(x) => write!(writer, "{}", x),
        Pattern::Oper(x) => write!(writer, "{}", x),
        Pattern::Tuple(lst) => {
            write!(writer, "(")?;
            for (i, pat) in lst.iter().enumerate() {
                if i != 0 {
                    write!(writer, ", ")?;
                }
                frestore_pattern(writer, pat)?;
            }
            write!(writer, ")")
        }
        Pattern::Constant(c) => frestore_constant(writer, c),
    }
}


/// Format an LlExpr
pub fn frestore_llexpr<W: Write>(writer: &mut W, exp: &LlExpr) -> core::fmt::Result<> {
    match exp {
        LlExpr::Constant(c) => frestore_constant(writer, c),
        LlExpr::Identifier(s) => write!(writer, "{}", s),
        LlExpr::Application { function, argument } => {
            write!(writer, "(")?;
            frestore_llexpr(writer, function)?;
            write!(writer, " ")?;
            frestore_llexpr(writer, argument)?;
            write!(writer, ")")
        }
        LlExpr::IfThenElse { condition, then_branch, else_branch } => {
            write!(writer, "(if (")?;
            frestore_llexpr(writer, condition)?;
            write!(writer, ") then ")?;
            frestore_llexpr(writer, then_branch)?;
            write!(writer, " else (")?;
            frestore_llexpr(writer, else_branch)?;
            write!(writer, "))")
        }
        LlExpr::LetIn { name, body, in_body, .. } => {
            write!(writer, "(let ")?;
            frestore_pattern(writer, name)?;
            write!(writer, " = ")?;
            frestore_llexpr(writer, body)?;
            write!(writer, " in ")?;
            frestore_llexpr(writer, in_body)?;
            write!(writer, ")")
        }
        LlExpr::Tuple(lst) => {
            write!(writer, "(")?;
            for (i, exp) in lst.iter().enumerate() {
                if i != 0 {
                    write!(writer, ", ")?;
                }
                frestore_llexpr(writer, exp)?;
            }
            write!(writer, ")")
        }
        LlExpr::Match { expr, branches } => {
            write!(writer, "(match ")?;
            frestore_llexpr(writer, expr)?;
            write!(writer, " with ")?;
            for (pat, exp) in branches {
                writeln!(writer, "| ")?;
                frestore_pattern(writer, pat)?;
                write!(writer, " -> ")?;
                frestore_llexpr(writer, exp)?;
            }
            write!(writer, ")")
        }
    }
}

/// Format a LlLetDeclaration
pub fn pp_lllet_declaration<W: Write + std::fmt::Display>(writer: &mut W, decl: &LlLetDeclaration) -> core::fmt::Result<> {
    match decl {
        LlLetDeclaration::DSingleLet { body, .. } => {
            write!(writer, "let ")?;
            write!(writer, " ")?;

            match body {
                crate::ll::ll_ast::LlBinding::Let { name, args, body } => {
                    frestore_pattern(writer, name)?;
                    for arg in args {
                        write!(writer, " ")?;
                        frestore_pattern(writer, arg)?;
                    }
                    write!(writer, " = ")?;
                    frestore_llexpr(writer, body)?;
                }
            }
        }
    }
    Ok(())
}

/// Format an entire LlProgram
pub fn pp_llprogram<W: Write + std::fmt::Display>(writer: &mut W, program: &LlProgram) -> core::fmt::Result<> {
    for decl in &program.0 {
        pp_lllet_declaration(writer, decl)?;
        writeln!(writer)?;
    }
    Ok(())
}