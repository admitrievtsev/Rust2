use crate::ast::{Expr, LetDeclaration, Pattern, SingleLet};
use std::collections::{BTreeSet, HashMap};

type Cont = fn(&ClConverter, &BTreeSet<String>, &HashMap<String, BTreeSet<String>>, &BTreeSet<String>, &Expr) -> Expr;

// Govnocode: This code is not idiomatic Rust. Probably I should move global_env and local_env in
// self boxed state like I did in ANF, but I have no time to rewrite it.
pub(crate) struct ClConverter {}

impl ClConverter {
    pub fn new() -> Self {
        ClConverter {}
    }

    #[warn(clippy::only_used_in_recursion)]
    pub fn global_names(pat: &Pattern) -> BTreeSet<String> {
        let res = match pat {
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
            Pattern::WildCard | Pattern::Constant(_) => BTreeSet::new(),
            Pattern::Cons(pat1, pat2) => {
                let set1 = Self::global_names(pat1);
                let set2 = Self::global_names(pat2);
                set1.union(&set2).cloned().collect()
            }
            Pattern::Tuple(pats) => {
                pats.iter()
                    .fold(BTreeSet::new(), |acc, pat| {
                        acc.union(&Self::global_names(pat)).cloned().collect()
                    })
            }
        };
        res
    }

    pub fn cc_function(&self, lts: &BTreeSet<String>, local_ctx: &HashMap<String, BTreeSet<String>>, global_ctx: &BTreeSet<String>, convert: Cont, expr: &Expr) -> Expr {
        match expr {
            Expr::Function(pat, body) => {
                Expr::Function(pat.clone(), Box::new(self.cc_function(lts, local_ctx, global_ctx, convert, body)))
            }
            _ => convert(self, lts, local_ctx, global_ctx, expr),
        }
    }

    fn cc_helper(&self,
                 lts: &BTreeSet<String>,
                 local_env: &HashMap<String, BTreeSet<String>>,
                 global_env: &BTreeSet<String>,
                 expr: &Expr,
    ) -> Expr {
        let stdlib_names: BTreeSet<String> = [
            "print_int".to_string(),
            "+".to_string(),
            "*".to_string(),
            "-".to_string(),
            "==".to_string(),
            "=".to_string(),
            "/".to_string(),
        ].iter().cloned().collect();
        match expr {
            Expr::Constant(constant) => Expr::Constant(constant.clone()),
            Expr::Identifier(id) => {
                match local_env.get(id) {
                    Some(free) => {
                        let mut result = Expr::Identifier(id.clone());
                        for free_id in free {
                            result = Expr::Application(
                                Box::new(result),
                                Box::new(Expr::Identifier(free_id.clone())),
                            );
                        }
                        result
                    }
                    None => Expr::Identifier(id.clone()),
                }
            }
            Expr::Function(pat, body) => {
                let unbound_names = self.free_ids(&Expr::Function(pat.clone(), body.clone()));
                let unbound_names_without_global: BTreeSet<String> = unbound_names
                    .difference(global_env)
                    .cloned()
                    .collect();
                let unbound_names_without_global: BTreeSet<String> = unbound_names_without_global
                    .difference(&stdlib_names)
                    .cloned()
                    .collect();

                let closed_fun = self.cc_function(
                    lts,
                    local_env,
                    &global_env.clone().difference(&stdlib_names)
                        .cloned().collect::<BTreeSet<_>>(),
                    ClConverter::cc_helper,
                    &Expr::Function(pat.clone(), body.clone()),
                );
                let mut result = closed_fun;
                let mut r_unbound: Vec<_> = unbound_names_without_global.clone().into_iter().collect();
                r_unbound.reverse();
                for unbound_id in r_unbound.iter() {
                    result = Expr::Function(
                        Pattern::Identifier(unbound_id.clone()),
                        Box::new(result),
                    );
                }
                let mut final_result = result;
                for unbound_id in unbound_names_without_global.iter() {
                    final_result = Expr::Application(
                        Box::new(final_result),
                        Box::new(Expr::Identifier(unbound_id.clone())),
                    );
                }

                final_result
            }
            Expr::Application(left, right) => {
                let new_left = self.cc_helper(lts, local_env, &global_env.difference(&stdlib_names)
                    .cloned().collect::<BTreeSet<_>>(), left);
                let new_right = self.cc_helper(lts, local_env, &global_env.difference(&stdlib_names)
                    .cloned().collect::<BTreeSet<_>>(), right);
                Expr::Application(Box::new(new_left), Box::new(new_right))
            }
            Expr::IfThenElse { condition, then_branch, else_branch } => {
                let new_condition = self.cc_helper(lts, local_env, &global_env.difference(&stdlib_names)
                    .cloned().collect::<BTreeSet<_>>(), condition);
                let new_then = self.cc_helper(lts, local_env, &global_env.difference(&stdlib_names)
                    .cloned().collect::<BTreeSet<_>>(), then_branch);
                let new_else = else_branch.as_ref().map(|e| Box::new(self.cc_helper(lts, local_env, &global_env.difference(&stdlib_names)
                    .cloned().collect::<BTreeSet<_>>(), e)));
                Expr::IfThenElse {
                    condition: Box::new(new_condition),
                    then_branch: Box::new(new_then),
                    else_branch: new_else,
                }
            }
            Expr::LetIn { is_rec, name, body, in_expr } => {
                match (name.as_ref(), body.as_ref()) {
                    (Pattern::Identifier(id), Expr::Function(_, _)) => {
                        let updated_lts = {
                            let mut set = lts.clone();
                            set.insert(id.clone());
                            set
                        };

                        let updated_global_env = {
                            let mut set = global_env.difference(&stdlib_names)
                                .cloned().collect::<BTreeSet<_>>().clone();
                            set.insert(id.clone());
                            set
                        };

                        let unbound_names = self.free_ids(&Expr::LetIn {
                            is_rec: *is_rec,
                            name: name.clone(),
                            body: body.clone(),
                            in_expr: in_expr.clone(),
                        });

                        let unbound_names_without_global: BTreeSet<String> = unbound_names
                            .difference(&updated_global_env)
                            .cloned()
                            .collect::<BTreeSet<_>>().difference(&stdlib_names)
                            .cloned().collect::<BTreeSet<_>>().clone();

                        let closed_fun = self.cc_function(
                            &updated_lts,
                            local_env,
                            &updated_global_env,
                            ClConverter::cc_helper,
                            body.as_ref(),
                        );

                        let unbound_ids_without_global: Vec<Pattern> = unbound_names_without_global
                            .iter()
                            .map(|x| Pattern::Identifier(x.clone()))
                            .collect();

                        let closed_outer = unbound_ids_without_global
                            .into_iter()
                            .rev()
                            .fold(closed_fun, |acc, pat| {
                                Expr::Function(pat, Box::new(acc))
                            });

                        let updated_local_env = {
                            let mut map = local_env.clone();
                            map.insert(id.clone(), unbound_names_without_global);
                            map
                        };

                        let closed_inner = self.cc_helper(
                            &updated_lts,
                            &updated_local_env,
                            global_env,
                            in_expr,
                        );

                        let updated_outer = self.cc_helper(
                            &updated_lts,
                            &updated_local_env,
                            &global_env.difference(&stdlib_names)
                                .cloned().collect::<BTreeSet<_>>().clone(),
                            &closed_outer,
                        );

                        Expr::LetIn {
                            is_rec: *is_rec,
                            name: Box::new(Pattern::Identifier(id.clone())),
                            body: Box::new(updated_outer),
                            in_expr: Box::new(closed_inner),
                        }
                    }
                    _ => {
                        Expr::LetIn {
                            is_rec: *is_rec,
                            name: name.clone(),
                            body: Box::new(self.cc_helper(lts, local_env, &global_env.difference(&stdlib_names)
                                .cloned().collect::<BTreeSet<_>>(), body)),
                            in_expr: Box::new(self.cc_helper(lts, local_env, &global_env.difference(&stdlib_names)
                                .cloned().collect::<BTreeSet<_>>(), in_expr)),
                        }
                    }
                }
            }
            Expr::Tuple(exps) => {
                let new_exps: Vec<Expr> = exps.iter()
                    .map(|exp| self.cc_helper(lts, local_env, global_env, exp))
                    .collect();
                Expr::Tuple(new_exps)
            }
            Expr::Match(expr, branches) => {
                let new_expr = self.cc_helper(lts, local_env, &global_env.difference(&stdlib_names)
                    .cloned().collect::<BTreeSet<_>>(), expr);
                let new_branches: Vec<(Pattern, Expr)> = branches.iter()
                    .map(|(pat, exp)| {
                        (pat.clone(), self.cc_helper(lts, local_env, global_env, exp))
                    })
                    .collect();
                Expr::Match(Box::new(new_expr), new_branches)
            }
            Expr::Unit => Expr::Unit,
        }
    }
    pub fn free_ids(&self, expr: &Expr) -> BTreeSet<String> {
        fn helper(expr: &Expr, bound_vars: &BTreeSet<String>) -> BTreeSet<String> {
            match expr {
                Expr::Constant(_) => BTreeSet::new(),
                Expr::Identifier(id) => {
                    if bound_vars.contains(id) {
                        BTreeSet::new()
                    } else {
                        let mut set = BTreeSet::new();
                        set.insert(id.clone());
                        set
                    }
                }
                Expr::Function(pat, body) => {
                    let mut new_bound = bound_vars.clone();
                    bind_pattern(pat, &mut new_bound);

                    helper(body, &new_bound)
                }
                Expr::Application(left, right) => {
                    let unbound_in_left = helper(left, bound_vars);
                    let unbound_in_right = helper(right, bound_vars);
                    unbound_in_left.union(&unbound_in_right).cloned().collect()
                }
                Expr::IfThenElse { condition, then_branch, else_branch } => {
                    let unbound_in_cond = helper(condition, bound_vars);
                    let unbound_in_then = helper(then_branch, bound_vars);
                    let unbound_in_else = match else_branch {
                        Some(e) => helper(e, bound_vars),
                        None => BTreeSet::new(),
                    };
                    unbound_in_cond
                        .union(&unbound_in_then)
                        .cloned()
                        .collect::<BTreeSet<_>>()
                        .union(&unbound_in_else)
                        .cloned()
                        .collect()
                }
                Expr::LetIn { name, body, in_expr, .. } => {
                    // For let expressions, we need to handle the binding properly
                    let mut new_bound = bound_vars.clone();
                    bind_pattern(name, &mut new_bound);

                    // Handle the outer expression
                    let unbound_in_body = helper(body, bound_vars);

                    // Handle the inner expression
                    let unbound_in_in_expr = helper(in_expr, &new_bound);

                    // Combine results
                    unbound_in_body.union(&unbound_in_in_expr).cloned().collect()
                }
                Expr::Tuple(exps) => {
                    exps.iter()
                        .fold(BTreeSet::new(), |acc, exp| {
                            acc.union(&helper(exp, bound_vars)).cloned().collect()
                        })
                }
                Expr::Match(expr, branches) => {
                    let unbound_in_expr = helper(expr, bound_vars);
                    let unbound_in_branches = branches.iter().fold(BTreeSet::new(), |acc, (pat, exp)| {
                        let mut new_bound = bound_vars.clone();
                        bind_pattern(pat, &mut new_bound);
                        acc.union(&helper(exp, &new_bound)).cloned().collect()
                    });
                    unbound_in_expr.union(&unbound_in_branches).cloned().collect()
                }
                Expr::Unit => BTreeSet::new(),
            }
        }

        fn bind_pattern(pat: &Pattern, bound_vars: &mut BTreeSet<String>) {
            match pat {
                Pattern::WildCard => {}
                Pattern::Cons(pat1, pat2) => {
                    bind_pattern(pat1, bound_vars);
                    bind_pattern(pat2, bound_vars);
                }
                Pattern::Identifier(id) => {
                    bound_vars.insert(id.clone());
                }
                Pattern::Oper(op) => {
                    bound_vars.insert(op.clone());
                }
                Pattern::Tuple(patterns) => {
                    for pat in patterns {
                        bind_pattern(pat, bound_vars);
                    }
                }
                Pattern::Constant(_) => {}
            }
        }

        helper(expr, &BTreeSet::new())
    }

    fn cc_decl(&self, global_ctx: &BTreeSet<String>, declaration: &LetDeclaration) -> LetDeclaration {
        let stdlib_names: BTreeSet<String> = [
            "print_int".to_string(),
            "+".to_string(),
            "::".to_string(),
            "*".to_string(),
            "-".to_string(),
            "==".to_string(),
            "=".to_string(),
            "/".to_string(),
        ].iter().cloned().collect();
        match declaration {
            LetDeclaration::SingleLet { is_rec, single_let } => {
                LetDeclaration::SingleLet {
                    is_rec: *is_rec,
                    single_let: SingleLet {
                        name: single_let.name.clone(),
                        binding: Box::new(self.cc_function(
                            &BTreeSet::new(),
                            &HashMap::new(),
                            &global_ctx.difference(&stdlib_names)
                                .cloned().collect::<BTreeSet<_>>(),
                            ClConverter::cc_helper,
                            single_let.binding.as_ref(),
                        )),
                    },
                }
            }
        }
    }


    pub fn convert(&self, global_ctx: &BTreeSet<String>, declaration: &LetDeclaration) -> LetDeclaration {
        let stdlib_names: BTreeSet<String> = [
            "print_int".to_string(),
            "+".to_string(),
            "-".to_string(),
            "*".to_string(),
            "==".to_string(),
            "=".to_string(),
            "/".to_string(),
        ].iter().cloned().collect();


        self.cc_decl(&global_ctx.difference(&stdlib_names)
            .cloned().collect::<BTreeSet<_>>(), declaration)
    }

    pub fn cc_ast(&self, ast: &[LetDeclaration]) -> Vec<LetDeclaration> {
        let mut converted = Vec::new();

        for item in ast {
            let mut global_ctx: BTreeSet<String> = BTreeSet::new();
            match item {
                LetDeclaration::SingleLet { single_let, .. } => {
                    let converted_item = self.convert(&global_ctx, item);
                    converted.push(converted_item);

                    let names = Self::global_names(single_let.name.as_ref());
                    global_ctx.extend(names);
                }
            }
        }

        converted
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_unbound_identifiers_simple() {
        let cc = ClConverter::new();
        let expr = Expr::Identifier("x".to_string());
        let result = cc.free_ids(&expr);
        assert!(result.contains("x"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_unbound_identifiers_function() {
        let expr = Expr::Function(
            Pattern::Identifier("x".to_string()),
            Box::new(Expr::Identifier("y".to_string())),
        );
        let cc = ClConverter::new();
        let result = cc.free_ids(&expr);
        assert!(result.contains("y"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_unbound_identifiers_application() {
        // Test application with two unbound identifiers
        let expr = Expr::Application(
            Box::new(Expr::Identifier("f".to_string())),
            Box::new(Expr::Identifier("x".to_string())),
        );

        let cc = ClConverter::new();
        let result = cc.free_ids(&expr);
        assert!(result.contains("f"));
        assert!(result.contains("x"));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_closure_conversion_simple() {
        // Parse a simple program
        let input = "let main = let x = 42 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn test_closure_conversion_function() {
        // Parse a function definition
        let input = "let main = let f = fun x -> x + 1 in f 5";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();


        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn test_closure_conversion_recursive() {
        // Parse a recursive function
        let input = "let rec fac n = if n <= 1 then 1 else n * fac (n - 1) let main = fac 5";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 2);
    }

    #[test]
    fn test_closure_conversion_nested() {
        // Parse a nested expression
        let input = "let main p = let x = 1 in let y = 2 in let z t u = t + x + y in z";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        println!("{:#?}", program.items);
        println!("{:#?}", converted);
        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn test_closure_conversion_complex() {
        // Parse a complex expression with multiple constructs
        let input = "let main = let x = 1 in let y = 2 in let z = if true then x else y in z + 1";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();


        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn test_closure_conversion_with_if_then_else() {
        // Parse an if-then-else expression
        let input = "let main = let x = if true then 1 else 2 in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn test_closure_conversion_with_tuple() {
        // Parse a tuple expression
        let input = "let main = let x = (1, 2) in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();


        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn test_closure_conversion_with_application() {
        // Parse an application expression
        let input = "let main = let f = fun x -> x + 1 in f 5";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn test_closure_conversion_with_let_in() {
        // Parse a let-in expression
        let input = "let main = let x = 1 in let y = x + 1 in y";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();


        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn test_closure_conversion_empty_program() {
        // Parse an empty program
        let input = "";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        // Convert the AST

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Should have no items
        assert_eq!(converted.len(), 0);
    }

    #[test]
    fn test_closure_conversion_multiple_declarations() {
        // Parse a program with multiple declarations
        let input = "let main = let x = 1 in let y = 2 in x + y let other = let z = 3 in z";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();


        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have both declarations
        assert_eq!(converted.len(), 2);
    }

    #[test]
    fn test_closure_conversion_with_complex_expression() {
        // Parse a complex expression
        let input = "let main = let x = (1 + 2) * (3 - 4) in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();


        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn test_closure_conversion_with_nested_functions() {
        // Parse a nested function
        let input = "let main = let f = fun x -> fun y -> x + y in f 1 2";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();


        let cc = ClConverter::new();
        // Convert the AST
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn test_closure_conversion_with_recursion() {
        // Parse a recursive function
        let input = "let rec fib n = if n <= 1 then n else fib (n - 1) + fib (n - 2) let main = fib 5";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        // Convert the AST

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 2);
    }

    #[test]
    fn test_closure_conversion_with_unit() {
        // Parse a unit expression
        let input = "let main = let x = () in x";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();

        // Convert the AST

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Should have the same number of items
        assert_eq!(converted.len(), 1);
    }
}