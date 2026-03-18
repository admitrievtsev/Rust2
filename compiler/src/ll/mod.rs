use crate::ast::{Expr, LetDeclaration, Pattern};
use std::collections::{BTreeSet, HashMap};

pub(crate) mod ll_ast;
pub(crate) mod ll_pp;


fn collect_bindings_from_pat(pat: &Pattern) -> BTreeSet<String> {
    match pat {
        Pattern::WildCard => BTreeSet::new(),
        Pattern::Constant(_) => BTreeSet::new(),
        Pattern::Identifier(id) => {
            let mut set = BTreeSet::new();
            set.insert(id.clone());
            set
        }
        Pattern::Oper(id) => {
            let mut set = BTreeSet::new();
            set.insert(id.clone());
            set
        }
        Pattern::Cons(left, right) => {
            let mut collected_in_left = collect_bindings_from_pat(left);
            let collected_in_right = collect_bindings_from_pat(right);
            collected_in_left.extend(collected_in_right);
            collected_in_left
        }
        Pattern::Tuple(pats) => {
            pats.iter().fold(BTreeSet::new(), |mut acc, pat| {
                let bindings = collect_bindings_from_pat(pat);
                acc.extend(bindings);
                acc
            })
        }
    }
}

fn new_name(env: &mut BTreeSet<String>, counter: i32) -> String {
    let name_candidate = format!("ll_{}", counter);
    if env.contains(&name_candidate) {
        new_name(env, counter + 1)
    } else {
        env.insert(name_candidate.clone());
        name_candidate
    }
}

fn collect_function_arguments(collected: Vec<Pattern>, expr: &Expr) -> (Vec<Pattern>, &Expr) {
    match expr {
        Expr::Function(pat, next) => {
            let mut new_collected = collected.clone();
            new_collected.push(pat.clone());
            collect_function_arguments(new_collected, next)
        }
        expr => {
            (collected, expr)
        }
    }
}

fn init_env(acc: BTreeSet<String>, prog: &[LetDeclaration]) -> BTreeSet<String> {
    prog.iter().fold(acc, |mut acc, decl| {
        match decl {
            LetDeclaration::SingleLet { is_rec: _, single_let } => {
                let bindings = collect_bindings_from_pat(&single_let.name);
                for pt in bindings.iter() {
                    acc.insert(pt.clone());
                }
                acc
            }
        }
    })
}

fn lift_expr(
    ctx: &mut HashMap<String, String>,
    acc: Vec<ll_ast::LlLetDeclaration>,
    global_ctx: &mut BTreeSet<String>,
    expr: &Expr,
) -> (ll_ast::LlExpr, Vec<ll_ast::LlLetDeclaration>) {
    match expr {
        Expr::Constant(constant) => {
            (ll_ast::LlExpr::Constant(constant.clone()), acc)
        }
        Expr::Identifier(id) => {
            match ctx.get(id) {
                Some(val) =>
                    (ll_ast::LlExpr::Identifier(val.clone()), acc),
                None =>
                    (ll_ast::LlExpr::Identifier(id.clone()), acc)
            }
        }
        Expr::IfThenElse { condition, then_branch, else_branch } => {
            let (lifted_guard, acc1) = lift_expr(ctx, acc, global_ctx, condition);
            let (lifted_if, acc2) = lift_expr(ctx, acc1, global_ctx, then_branch);
            let (lifted_else, acc3) = match else_branch {
                Some(else_expr) => lift_expr(ctx, acc2, global_ctx, else_expr),
                None => (ll_ast::LlExpr::Constant(crate::ast::Constant::Unit), acc2)
            };
            (
                ll_ast::LlExpr::IfThenElse {
                    condition: Box::new(lifted_guard),
                    then_branch: Box::new(lifted_if),
                    else_branch: Box::new(lifted_else),
                },
                acc3
            )
        }
        Expr::Application(left_exp, right_exp) => {
            let (lifted_left, acc1) = lift_expr(ctx, acc, global_ctx, left_exp);
            let (lifted_right, acc2) = lift_expr(ctx, acc1, global_ctx, right_exp);

            (
                ll_ast::LlExpr::Application {
                    function: Box::new(lifted_left),
                    argument: Box::new(lifted_right),
                },
                acc2,
            )
        }
        Expr::Tuple(exp_list) => {
            let mut lifted_exprs = Vec::new();
            let mut current_acc = acc;

            for e in exp_list {
                let (l, acc1) = lift_expr(ctx, current_acc, global_ctx, e);
                lifted_exprs.push(l);
                current_acc = acc1;
            }

            (
                ll_ast::LlExpr::Tuple(lifted_exprs),
                current_acc,
            )
        }
        Expr::Match(expr, branches) => {
            let (lifted_exp, acc1) = lift_expr(ctx, acc, global_ctx, expr);

            let mut lifted_branches = Vec::new();
            let mut current_acc = acc1;

            for (p, e) in branches {
                let (lifted_expr, acc2) = lift_expr(ctx, current_acc, global_ctx, e);
                lifted_branches.push((p.clone(), lifted_expr));
                current_acc = acc2;
            }

            (
                ll_ast::LlExpr::Match {
                    expr: Box::new(lifted_exp),
                    branches: lifted_branches,
                },
                current_acc,
            )
        }
        Expr::LetIn { is_rec, name, body, in_expr } => {
            match (name.as_ref(), body.as_ref()) {
                (Pattern::Identifier(id), Expr::Function(_, _)) => {
                    let (args, new_outer) = collect_function_arguments(vec![], body);

                    let fresh_name = new_name(global_ctx, 0);
                    let mut updated_ctx = ctx.clone();
                    updated_ctx.insert(id.clone(), fresh_name.clone());

                    let (lifted_outer, acc1) = match is_rec {
                        true => lift_expr(&mut updated_ctx, acc, global_ctx, new_outer),
                        false => lift_expr(ctx, acc, global_ctx, new_outer),
                    };

                    let mut res = vec![ll_ast::LlLetDeclaration::DSingleLet {
                        is_rec: *is_rec,
                        body: ll_ast::LlBinding::Let {
                            name: Pattern::Identifier(fresh_name.clone()),
                            args: args.clone(),
                            body: lifted_outer,
                        },
                    }];

                    res.extend(acc1);

                    let (lifted_inner, acc2) = lift_expr(
                        &mut updated_ctx,
                        res,
                        global_ctx,
                        in_expr,
                    );

                    (lifted_inner, acc2)
                }
                _ => {
                    let (lifted_outer, acc1) = lift_expr(ctx, acc, global_ctx, body);
                    let (lifted_inner, acc2) = lift_expr(ctx, acc1, global_ctx, in_expr);
                    (
                        ll_ast::LlExpr::LetIn {
                            is_rec: *is_rec,
                            name: name.as_ref().clone(),
                            body: Box::new(lifted_outer),
                            in_body: Box::new(lifted_inner),
                        },
                        acc2,
                    )
                }
            }
        }
        Expr::Function(pat, body) => {
            let exp = Expr::Function(pat.clone(), body.clone());
            let (arguments, new_body) = collect_function_arguments(vec![], &exp);

            let fresh_name = new_name(global_ctx, 0);

            let (lifted, acc1) = lift_expr(
                &mut HashMap::new(),
                acc,
                global_ctx,
                new_body,
            );

            let mut res = vec![
                ll_ast::LlLetDeclaration::DSingleLet {
                    is_rec: false,
                    body: ll_ast::LlBinding::Let {
                        name: Pattern::Identifier(fresh_name.clone()),
                        args: arguments,
                        body: lifted,
                    },
                }
            ];
            res.extend(acc1);

            (
                ll_ast::LlExpr::Identifier(fresh_name),
                res,
            )
        }
        Expr::Unit => {
            (ll_ast::LlExpr::Constant(crate::ast::Constant::Unit), acc)
        }
    }
}

fn lift_bindings(
    global_ctx: &mut BTreeSet<String>,
    decl: &LetDeclaration,
) -> Vec<ll_ast::LlLetDeclaration> {
    match decl {
        LetDeclaration::SingleLet { is_rec, single_let } => {
            let (pats, expr) = collect_function_arguments(vec![], &single_let.binding);

            let (lifted, acc) = lift_expr(
                &mut HashMap::new(),
                vec![],
                global_ctx,
                expr,
            );

            let mut res = vec![
                ll_ast::LlLetDeclaration::DSingleLet {
                    is_rec: *is_rec,
                    body: ll_ast::LlBinding::Let {
                        name: single_let.name.as_ref().clone(),
                        args: pats,
                        body: lifted,
                    },
                }
            ];
            res.extend(acc);

            res
        }
    }
}

fn prog_lift(prog: &[LetDeclaration]) -> Vec<ll_ast::LlLetDeclaration> {
    let lifted = prog.iter().fold(vec![], |h, decl| {
        let mut global_ctx = init_env(BTreeSet::new(), prog);
        let mut new_acc = lift_bindings(&mut global_ctx, decl);
        new_acc.reverse();
        h.iter().chain(new_acc.iter()).cloned().collect()
    });

    lifted
}

pub fn lift_ast(prog: &crate::ast::Program) -> ll_ast::LlProgram {
    let lifted = prog_lift(&prog.items);
    ll_ast::LlProgram(lifted)
}

#[cfg(test)]
mod tests {
    use crate::ast::Program;
    use crate::cc::ClConverter;
    use crate::ll::lift_ast;
    use crate::ll::ll_pp::pp_llprogram;
    use crate::parser::Parser;

    #[test]
    fn test_simple_constant_with_parser() {
        // Parse and test simple constant expression
        let input = "let main = let x = 42 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        let result = lift_ast(&program);

        assert_eq!(result.0.len(), 1);
    }

    #[test]
    fn test_simple_function_with_parser() {
        // Parse and test simple function expression
        let input = "let main = let f = fun x -> 42 in f 1";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        let result = lift_ast(&program);
        assert_eq!(result.0.len(), 2);
    }

    #[test]
    fn test_complex_expression_with_parser() {
        // Parse and test more complex expression
        let input = "let main = let rec fac n = if n <= 1 then 1 else n * fac (n - 1) in fac 5";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        let result = lift_ast(&program);
        assert_eq!(result.0.len(), 2);
    }

    #[test]
    fn test_integration_with_cc() {
        // Test integration between parsing, lifting, and CC conversion
        let input = "let main p = let x = 1 in let y r = 2 in let z t u = t + x + y + p + u + r in z";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        // Then convert to CC form
        let cc = ClConverter::new();
        let cc_result = cc.cc_ast(&program.items);
        // First lift the AST
        let lifted = lift_ast(&Program { items: cc_result.clone() });

        let mut stdout = std::string::String::new();
        pp_llprogram(&mut stdout, &lifted).unwrap();
        // println!("{:#?}", lifted);
        println!("{}", stdout);
        // Both should succeed without panics
        assert!(!lifted.0.is_empty());
        assert!(!cc_result.is_empty());
    }

    #[test]
    fn test_simple_full() {
        // Test integration between parsing, lifting, and CC conversion
        let input = "let rec fix f x = f (fix f) x
                           let fac self n = if n<=1 then 1 else n * self (n-1)";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:#?}", program);
        // Then convert to CC form
        let cc = ClConverter::new();
        let cc_result = cc.cc_ast(&program.items);
        println!("{:#?}", cc_result);
        // First lift the AST
        let lifted = lift_ast(&Program { items: cc_result.clone() });

        let mut ll_ref = String::new();
        pp_llprogram(&mut ll_ref, &lifted).unwrap();
        println!("{:#?}", ll_ref);
        // Both should succeed without panics
        assert!(!lifted.0.is_empty());
        assert!(!cc_result.is_empty());
    }

    #[test]
    fn test_fac_full() {
        // Test integration between parsing, lifting, and CC conversion
        let input = "let rec fac n = if n <= 1 then 1 else n * fac (n - 1) let main = fac 4 2";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:#?}", program);
        // Then convert to CC form
        let cc = ClConverter::new();
        let cc_result = cc.cc_ast(&program.items);
        println!("{:#?}", cc_result);
        // First lift the AST
        let lifted = lift_ast(&Program { items: cc_result.clone() });

        let mut stdout = std::string::String::new();
        pp_llprogram(&mut stdout, &lifted).unwrap();
        // println!("{:#?}", lifted);
        println!("{:#?}", stdout);
        // Both should succeed without panics
        assert!(!lifted.0.is_empty());
        assert!(!cc_result.is_empty());
    }

    #[test]
    fn test_apply_full() {
        let input = "let p = print_int ((y 6) + 1 + 2)";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:#?}", program);
        let cc = ClConverter::new();
        let cc_result = cc.cc_ast(&program.items);
        println!("{:#?}", cc_result);
        let lifted = lift_ast(&Program { items: cc_result.clone() });

        let mut stdout = std::string::String::new();
        pp_llprogram(&mut stdout, &lifted).unwrap();
        // println!("{:#?}", lifted);
        println!("{:#?}", stdout);
        assert!(!lifted.0.is_empty());
        assert!(!cc_result.is_empty());
    }
}