use crate::anf::ast_anf::*;

pub fn list_to_string<T, F>(pp: F, sep: &str, lst: &[T]) -> String
where
    F: Fn(&T) -> String,
{
    match lst {
        [] => "".to_string(),
        [x] => pp(x),
        lst => {
            let mut iter_tmp = lst.iter();
            let x = iter_tmp.next().unwrap();
            let xs = iter_tmp.collect::<Vec<_>>();

            let mut result = pp(x);
            for x in &xs[1..] {
                result = format!("{}{}{}", result, sep, pp(x));
            }
            result
        }
    }
}

pub fn imm_to_string(imm: &ImmExpr) -> String {
    match imm {
        ImmExpr::Int(i) => i.to_string(),
        ImmExpr::Bool(false) => "false".to_string(),
        ImmExpr::Bool(true) => "true".to_string(),
        ImmExpr::Nil => "[]".to_string(),
        ImmExpr::Identifier(id) => id.clone(),
        ImmExpr::Unit => "()".to_string(),
        ImmExpr::Tuple(tup) => {
            format!("({})", list_to_string(imm_to_string, ", ", tup))
        }
    }
}

pub fn cexpr_to_string(cexpr: &CExpr) -> String {
    match cexpr {
        CExpr::ImmExpr(imm) => imm_to_string(imm),
        CExpr::IfThenElse { condition, then_branch, else_branch } => {
            format!(
                "if {} then {} else {}",
                imm_to_string(condition),
                aexpr_to_string(then_branch),
                aexpr_to_string(else_branch)
            )
        }
        CExpr::Application(left, right, args) => {
            let args_str = args.iter().map(imm_to_string).collect::<Vec<_>>().join(" ");
            format!(
                "{} {} {}",
                imm_to_string(left),
                imm_to_string(right),
                args_str
            )
        }
    }
}

pub fn aexpr_to_string(aexpr: &AExpr) -> String {
    match aexpr {
        AExpr::CExpr(cexp) => cexpr_to_string(cexp),
        AExpr::LetIn { name, body, in_body } => {
            format!(
                "let {} = {} in\n{}",
                name,
                cexpr_to_string(body),
                aexpr_to_string(in_body)
            )
        }
    }
}

pub fn rec_flag_to_string(is_rec: bool) -> String {
    if is_rec {
        "rec".to_string()
    } else {
        "".to_string()
    }
}

pub fn anf_decl_to_string(decl: &AnfDecl) -> String {
    match decl {
        AnfDecl::SingleLet { is_rec, single_anf_binding } => {
            match single_anf_binding {
                SingleAnfBinding::Let { name, args, body } => {
                    format!(
                        "let {} {} {} = {};;",
                        rec_flag_to_string(*is_rec),
                        name,
                        args.join(" "),
                        aexpr_to_string(body)
                    )
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn program_to_string(declarations: &[AnfDecl]) -> String {
    declarations
        .iter()
        .map(anf_decl_to_string)
        .collect::<Vec<_>>()
        .join("\n")
}