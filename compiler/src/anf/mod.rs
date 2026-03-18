pub(crate) mod ast_anf;
pub(crate) mod anf_pp;

use crate::anf::ast_anf::{AExpr, AnfDecl, AnfProg, CExpr, ImmExpr, SingleAnfBinding};
use crate::ast::{Constant, Pattern};
use crate::ll::ll_ast::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// Prefix types for fresh name generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Prefix {
    IfThenElse,
    Tuple,
    Application,
    Constraint,
}

impl Prefix {
    fn as_string(&self) -> &'static str {
        match self {
            Prefix::IfThenElse => "anf_ifthenelse_",
            Prefix::Tuple => "anf_tuple_",
            Prefix::Application => "anf_app_",
            Prefix::Constraint => "anf_constraint_",
        }
    }
}

pub struct ANFConverter {
    state: RefCell<HashMap<Prefix, u32>>,
    infix_rename: RefCell<HashMap<Pattern, String>>,
    apply_inf: RefCell<HashMap<String, String>>,
    global: HashSet<String>,
}

impl Default for ANFConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl ANFConverter {
    pub fn new() -> Self {
        ANFConverter {
            state: RefCell::new(HashMap::new()),
            infix_rename: RefCell::new(HashMap::new()),
            apply_inf: RefCell::new(HashMap::new()),
            global: Default::default(),
        }
    }
    pub fn init(&self) {
        let mut guard = self.state.borrow_mut();
        guard.insert(Prefix::IfThenElse, 0);
        guard.insert(Prefix::Tuple, 0);
        guard.insert(Prefix::Application, 0);
        guard.insert(Prefix::Constraint, 0);
    }

    fn get_new_num(&self, prefix: Prefix) -> u32 {
        let mut guard = self.state.borrow_mut();
        let num = guard[&prefix];
        let new_num = num + 1;
        guard.insert(prefix, new_num);
        num
    }

    fn pattern_to_string(&self, pat: &Pattern) -> String {
        match pat {
            Pattern::Identifier(id) => id.clone(),
            Pattern::Oper(op) => match self.infix_rename.borrow().get(pat) {
                Some(new_op) => new_op.clone(),
                None => op.clone(),
            },
            _ => panic!("Only identifiers are supported in patterns"),
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn new_name(&self, prefix: Prefix, global: &HashSet<String>) -> String {
        let new_num = self.get_new_num(prefix);

        let name_candidate = format!("{}{}", prefix.as_string(), new_num);
        if self.global.contains(&name_candidate) {
            self.new_name(prefix, global)
        } else {
            name_candidate
        }
    }

    fn sep_llapp(expr: &LlExpr, rest: &mut Vec<LlExpr>) -> LlExpr {
        match expr {
            LlExpr::Application { function: left, argument: right } => {
                rest.push(*right.clone());
                Self::sep_llapp(left, rest)
            }
            expr => {
                rest.reverse();
                expr.clone()
            }
        }
    }

    fn anf(&self, ctx: &HashSet<String>, llexpr: &LlExpr, expr_with_hole: Box<dyn FnOnce(ImmExpr) -> AExpr + '_>) -> AExpr {
        match llexpr {
            LlExpr::Constant(constant) => {
                let imm_const = match constant {
                    Constant::Int(i) => ImmExpr::Int(*i),
                    Constant::Bool(b) => ImmExpr::Bool(*b),
                    Constant::Unit => ImmExpr::Unit,
                    Constant::Nil => ImmExpr::Nil,
                };
                expr_with_hole(imm_const)
            }
            LlExpr::Identifier(id) => {
                let new_ident = match self.apply_inf.borrow().get(id) {
                    Some(new_id) => new_id.clone(),
                    None => id.clone(),
                };
                expr_with_hole(ImmExpr::Identifier(new_ident.clone()))
            }
            LlExpr::IfThenElse { condition, then_branch, else_branch } => {
                self.anf(ctx, condition, Box::new(|guard| {
                    let then_aexpr = self.anf(ctx, then_branch, Box::new(|imm_then| {
                        AExpr::CExpr(Box::new(CExpr::ImmExpr(Box::new(imm_then))))
                    }));
                    let else_aexpr = self.anf(ctx, else_branch, Box::new(|imm_else| {
                        AExpr::CExpr(Box::new(CExpr::ImmExpr(Box::new(imm_else))))
                    }));

                    let fresh_name = self.new_name(Prefix::IfThenElse, ctx);
                    let imm_id = ImmExpr::Identifier(fresh_name.clone());
                    let aexp = expr_with_hole(imm_id);

                    AExpr::LetIn {
                        name: fresh_name,
                        body: Box::new(CExpr::IfThenElse {
                            condition: Box::new(guard),
                            then_branch: then_aexpr,
                            else_branch: else_aexpr,
                        }),
                        in_body: Box::new(aexp),
                    }
                }))
            }
            LlExpr::Tuple(elems) => {
                let mut acc = vec![];
                self.anf_list(ctx, &mut acc, &mut elems.iter(), Box::new(|imm: Vec<ImmExpr>| {
                    let fresh_name = self.new_name(Prefix::Tuple, ctx);
                    let imm_id = ImmExpr::Identifier(fresh_name.clone());
                    let aexp = expr_with_hole(imm_id);

                    AExpr::LetIn {
                        name: fresh_name,
                        body: Box::new(CExpr::ImmExpr(Box::new(ImmExpr::Tuple(imm.clone())))),
                        in_body: Box::new(aexp),
                    }
                }))
            }
            LlExpr::Application { function, argument } => {
                let mut rest = vec![];
                let llexp = Self::sep_llapp(&LlExpr::Application { function: function.clone(), argument: argument.clone() }, &mut rest);
                let rest = &mut rest.as_slice().iter();
                self.anf(ctx, &llexp, Box::new(|imm_exp| {
                    self.anf_list(ctx, &mut vec![], rest, Box::new(|mut imm_rest: Vec<ImmExpr>| {
                        let fresh_name = self.new_name(Prefix::Application, ctx);
                        let imm_id = ImmExpr::Identifier(fresh_name.clone());

                        imm_rest.reverse();
                        let built_app = match &imm_rest.clone()[..]
                        {
                            [] => panic!("Empty argument list. ll_exp: {:?} fresh_name: {fresh_name}, imm_id: {:?}", llexpr, imm_id.clone()),
                            [head] => CExpr::Application(Box::new(imm_exp), Box::new(head.clone()), Vec::new()),
                            _ => CExpr::Application(Box::new(imm_exp), Box::new(imm_rest[0].clone()), imm_rest[1..].to_vec())
                        };
                        AExpr::LetIn {
                            name: fresh_name,
                            body: Box::new(built_app),
                            in_body: Box::new(expr_with_hole(imm_id.clone())),
                        }
                    }))
                }))
            }
            LlExpr::LetIn { name, body, in_body, .. } => {
                let new_env = self.patern_bindings(name);

                self.anf(&new_env.clone(), body, Box::new(|imm_outer| {
                    self.anf(&new_env.clone(), in_body, Box::new(|aexp| {
                        AExpr::LetIn {
                            name: self.pattern_to_string(name),
                            body: Box::new(CExpr::ImmExpr(Box::new(imm_outer))),
                            in_body: Box::new(AExpr::CExpr(Box::new(CExpr::ImmExpr(Box::new(aexp))))),
                        }
                    }))
                }))
            }
            _ => {
                expr_with_hole(ImmExpr::Unit)
            }
        }
    }

    fn anf_list(&self,
                ctx: &HashSet<String>,
                acc: &mut Vec<ImmExpr>,
                llexprs: &mut std::slice::Iter<LlExpr>,
                coninuation_fn: Box<dyn FnOnce(Vec<ImmExpr>) -> AExpr + '_>)
                -> AExpr {
        let llexpr = llexprs.next();
        match llexpr {
            Some(llexpr) => {
                self.anf(ctx, llexpr, Box::new(|imm| {
                    acc.push(imm);
                    self.anf_list(ctx, acc, llexprs, coninuation_fn)
                }))
            }
            None => {
                acc.reverse();
                coninuation_fn(acc.clone())
            }
        }
    }

    fn patern_bindings(&self, pat: &Pattern) -> HashSet<String> {
        match pat {
            Pattern::WildCard => HashSet::new(),
            Pattern::Constant(_) => HashSet::new(),
            Pattern::Identifier(id) => {
                let mut set = HashSet::new();
                set.insert(id.clone());
                set
            }
            Pattern::Oper(id) => {
                let mut set = HashSet::new();
                let id = match self.infix_rename.borrow().get(pat) {
                    Some(new_op) => new_op.clone(),
                    None => id.clone(),
                };
                set.insert(id.clone());
                set
            }
            Pattern::Cons(left, right) => {
                let mut collected_in_left = self.patern_bindings(left);
                let collected_in_right = self.patern_bindings(right);
                collected_in_left.extend(collected_in_right);
                collected_in_left
            }
            Pattern::Tuple(pats) => {
                pats.iter().fold(HashSet::new(), |mut acc, pat| {
                    let bindings = self.patern_bindings(pat);
                    acc.extend(bindings);
                    acc
                })
            }
        }
    }


    fn anf_decl(&self, ctx: &HashSet<String>, decl: &LlLetDeclaration) -> Option<AnfDecl> {
        match decl {
            LlLetDeclaration::DSingleLet { is_rec, body } => {
                match body {
                    LlBinding::Let { name, args, body } => {
                        let collect_bindings = |acc: HashSet<String>, pat: &Pattern| -> HashSet<String> {
                            let bindings = self.patern_bindings(pat);
                            let mut new_acc = acc;
                            new_acc.extend(bindings);
                            new_acc
                        };

                        let new_args = args.iter().map(|pat| {
                            match pat {
                                Pattern::Identifier(id) => id.clone(),
                                _ => "error".to_string(),
                            }
                        }).collect::<Vec<_>>();

                        let aexpr = self.anf(&(collect_bindings(ctx.clone(), name)), body, Box::new(|imm_expr| {
                            AExpr::CExpr(Box::new(CExpr::ImmExpr(Box::new(imm_expr))))
                        }));
                        let anf_let = SingleAnfBinding::Let {
                            name: self.pattern_to_string(name),
                            args: new_args,
                            body: aexpr,
                        };

                        Some(AnfDecl::SingleLet {
                            is_rec: *is_rec,
                            single_anf_binding: anf_let,
                        })
                    }
                }
            }
        }
    }

    fn collect_bindings(&self, decls: &[LlLetDeclaration]) -> HashSet<String> {
        let mut result = HashSet::new();

        for decl in decls {
            match decl {
                LlLetDeclaration::DSingleLet { body, .. } => {
                    match body {
                        LlBinding::Let { name, .. } => {
                            let bindings = self.patern_bindings(name);
                            result.extend(bindings);
                        }
                    }
                }
            }
        }

        result
    }

    fn convert_decls(&self, decls: &[LlLetDeclaration], acc: Vec<AnfDecl>) -> Vec<AnfDecl> {
        let ctx = self.collect_bindings(decls);

        if decls.is_empty() {
            acc.into_iter().rev().collect()
        } else {
            let head = &decls[0];
            let tail = &decls[1..];

            let anf_decl_opt = self.anf_decl(&ctx, head);

            match anf_decl_opt {
                Some(anf_decl) => {
                    let new_acc = vec![anf_decl].into_iter().chain(acc).collect::<Vec<_>>();
                    self.convert_decls(tail, new_acc)
                }
                None => self.convert_decls(tail, acc)
            }
        }
    }

    pub fn transform_anf(&self, ll_prog: LlProgram) -> AnfProg {
        for item in ll_prog.0.iter() {
            match item {
                LlLetDeclaration::DSingleLet { body, .. } => {
                    match body {
                        LlBinding::Let { name, .. } => {
                            if let Pattern::Oper(id) = name {
                                let new_num = self.infix_rename.borrow().len();
                                self.infix_rename.borrow_mut().insert(name.clone(), format!("infix_{}", new_num));
                                self.apply_inf.borrow_mut().insert(format!("({})", id), format!("infix_{}", new_num));
                            };
                        }
                    }
                }
            }
        }
        let anf_decls = self.convert_decls(&ll_prog.0, Vec::new());

        AnfProg(anf_decls)
    }
}

#[cfg(test)]
mod tests {
    use crate::anf::anf_pp::program_to_string;
    use crate::anf::ANFConverter;
    use crate::ast::Program;
    use crate::cc::*;
    use crate::ll::lift_ast;
    use crate::parser::Parser;

    /// Test basic ANF transformation with simple expressions
    #[test]
    fn test_anf_simple_expression() {
        let input = "let main = 42";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        assert!(!result.is_empty());
    }

    /// Test basic ANF transformation with simple expressions
    #[test]
    fn test_anf_simple_expression_2() {
        let input = "let p = print_int ((y 6) + 1 + 2)";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:?}", program);
        let mut anf_converter = ANFConverter::new();
        anf_converter.init();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
    }

    /// Test basic ANF transformation with simple expressions
    #[test]
    fn test_anf_manytests04() {
        let input = "let test10 a b c d e f g h i j = a (b + c) d e (f (g h)) (i + j)";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        println!("{:#?}", program);
        let mut anf_converter = ANFConverter::new();
        anf_converter.init();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
    }

    #[test]
    fn test_anf_manytests05() {
        let input = "let rec fix f x = f (fix f) x
                           let fac self n = if n<=1 then 1 else n * self (n-1)";
        let mut parser = Parser::new(input);
        let program = parser.parse_program();
        let mut anf_converter = ANFConverter::new();
        anf_converter.init();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
    }


    /// Test factorial function in ANF
    #[test]
    fn test_anf_factorial() {
        let input = "let rec fac n = if n <= 1 then 1 else n * fac (n - 1) let main = fac 4";
        let mut parser = Parser::new(input);
        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        anf_converter.init();
        let program = parser.parse_program();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        assert!(!result.is_empty());
    }

    /// Test fibonacci function in ANF
    #[test]
    fn test_anf_fibonacci() {
        let input = "let rec fib n = if n <= 1 then n else fib (n - 1) + fib (n - 2) let main = fib 5";
        let mut parser = Parser::new(input);

        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let program = parser.parse_program();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        assert!(!result.is_empty());
    }

    /// Test simple let expression in ANF
    #[test]
    fn test_anf_simple_let() {
        let input = "let main = let x = 1 in x + 1";
        let mut parser = Parser::new(input);

        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let program = parser.parse_program();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        assert!(!result.is_empty());
    }

    /// Test if-then-else in ANF
    #[test]
    fn test_anf_if_then_else() {
        let input = "let main = if true then 1 else 2";
        let mut parser = Parser::new(input);

        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let program = parser.parse_program();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        assert!(!result.is_empty());
    }

    /// Test tuple expressions in ANF
    #[test]
    fn test_anf_tuples() {
        let input = "let main = (1, 2)";
        let mut parser = Parser::new(input);

        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let program = parser.parse_program();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        assert!(!result.is_empty());
    }

    /// Test nested function applications in ANF
    #[test]
    fn test_anf_nested_applications() {
        let input = "let main = let f = fun x -> x + 1 in f (f 1)";
        let mut parser = Parser::new(input);

        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let program = parser.parse_program();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        assert!(!result.is_empty());
    }

    /// Test multiple declarations in ANF
    #[test]
    fn test_anf_multiple_declarations() {
        let input = "let main = let x = 1 in let y = 2 in x + y";
        let mut parser = Parser::new(input);

        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let program = parser.parse_program();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        assert!(!result.is_empty());
    }

    /// Test recursive mutual declarations in ANF
    #[test]
    fn test_anf_mutual_recursion() {
        let input = "let rec f x = g x let rec g x = f x let main = f 1";
        let mut parser = Parser::new(input);

        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let program = parser.parse_program();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        assert!(!result.is_empty());
    }

    /// Test complex factorial with multiple let bindings
    #[test]
    fn test_anf_complex_factorial() {
        let input = "let rec fac n =
            let one = 1 in
            if n <= one then one
            else n * fac (n - one)
        let main = fac 4";
        let mut parser = Parser::new(input);

        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let program = parser.parse_program();

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&program.items);

        // Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Convert to ANF
        let anf_program = anf_converter.transform_anf(ll_program);

        // Check the result using pretty printer
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_full_pipeline() {
        let input = "let rec fac n = if n <= 1 then 1 else n * fac (n - 1) let main = fac 4";

        // Step 1: Parse to AST
        let mut parser = Parser::new(input);

        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let ast_program = parser.parse_program();

        // Step 2: Convert AST to closure-converted form

        let cc = ClConverter::new();
        let converted = cc.cc_ast(&ast_program.items);

        // Step 3: Convert to LL form
        let ll_program = lift_ast(&Program { items: converted });

        // Step 4: Convert to ANF form
        let anf_program = anf_converter.transform_anf(ll_program);

        // Step 5: Pretty print the result
        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        // Basic checks
        assert!(!result.is_empty());
        assert!(result.contains("let"));
        assert!(result.contains("="));
    }

    #[test]
    fn test_manyargs() {
        let input = "let wrap f = if 1 = 1 then f else f
            let test3 a b c =
              let a = print_int a in
              let b = print_int b in
              let c = print_int c in
              0

            let test10 a b c d e f g h i j = a + b + c + d + e + f + g + h + i + j

            let main =
              let rez =
                  (wrap test10 1 10 100 1000 10000 100000 1000000 10000000 100000000
                     1000000000)
              in
              let () = print_int rez in
              let temp2 = wrap test3 1 10 100 in
              0";

        // Step 1: Parse to AST
        let mut parser = Parser::new(input);

        let mut anf_converter = ANFConverter::new();
        anf_converter.init();
        let ast_program = parser.parse_program();


        let cc = ClConverter::new();
        let converted = cc.cc_ast(&ast_program.items);

        let ll_program = lift_ast(&Program { items: converted });

        let anf_program = anf_converter.transform_anf(ll_program);

        let result = program_to_string(&anf_program.0);
        println!("{}", result);
        assert!(!result.is_empty());
        assert!(result.contains("let"));
        assert!(result.contains("="));
    }
}