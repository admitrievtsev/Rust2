// src/codegen/mod.rs

use crate::anf::ast_anf::ImmExpr::*;
use crate::anf::ast_anf::{AExpr, AnfDecl, AnfProg, CExpr, ImmExpr, SingleAnfBinding};
use crate::ast::TypeName;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::llvm_sys::LLVMTailCallKind;
use inkwell::module::Module;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{InitializationConfig, Target, TargetMachineOptions};
use inkwell::types::{BasicMetadataTypeEnum, FunctionType, IntType};
use inkwell::values::{AnyValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum, CallSiteValue, FunctionValue, GlobalValue, IntValue};
use inkwell::IntPredicate;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LLVMName(pub String);

impl LLVMName {
    pub fn get_full_name(&self) -> String {
        format!("{}_llvm", self.0)
    }
    pub fn get_name(&self) -> String {
        self.0.to_string()
    }
}

impl From<&str> for LLVMName {
    fn from(s: &str) -> Self {
        let std_func = get_std_funs();
        if std_func.contains_key(s) {
            return LLVMName(std_func[s].llvm_name.clone());
        }
        LLVMName(s.to_string())
    }
}

pub fn get_global_name(name: &str) -> String {
    format!("{}_global", name)
}


impl From<String> for LLVMName {
    fn from(s: String) -> Self {
        let std_func = get_std_funs();
        if std_func.contains_key(s.clone().as_str()) {
            return LLVMName(std_func[s.as_str()].llvm_name.clone());
        };
        LLVMName(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsSystem {
    SystemFun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsVararg {
    NotVararg,
    Vararg,
}

#[derive(Debug, Clone)]
pub struct FunType(pub IsSystem, pub IsVararg);

#[derive(Debug, Clone)]
pub struct StdFun {
    name: String,
    pub llvm_name: String,
    pub fun_type: FunType,
    pub tp: TypeName,
}

pub const RT_TYPE: FunType = FunType(IsSystem::SystemFun, IsVararg::NotVararg);

pub fn bin_op(arg1: TypeName, arg2: TypeName, ret: TypeName) -> TypeName {
    TypeName::Function(Box::new(arg1), Box::new(TypeName::Function(Box::new(arg2), Box::new(ret))))
}


// TODO Govnokod replacement better LLVNName linking
pub fn get_std_funs() -> BTreeMap<String, StdFun> {
    BTreeMap::from([
        ("(+)".to_string(), StdFun {
            name: "(+)".to_string(),
            llvm_name: "plus_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Int, TypeName::Int, TypeName::Int),
        }),
        ("(-)".to_string(), StdFun {
            name: "(-)".to_string(),
            llvm_name: "sub_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Int, TypeName::Int, TypeName::Int),
        }),
        ("(*)".to_string(), StdFun {
            name: "(*)".to_string(),
            llvm_name: "mul_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Int, TypeName::Int, TypeName::Int),
        }),
        ("(/)".to_string(), StdFun {
            name: "(/)".to_string(),
            llvm_name: "div_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Int, TypeName::Int, TypeName::Int),
        }),
        ("(<)".to_string(), StdFun {
            name: "(<)".to_string(),
            llvm_name: "l_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Poly("'_a".to_string()), TypeName::Poly("'_a".to_string()), TypeName::Bool),
        }),
        ("(<=)".to_string(), StdFun {
            name: "(<=)".to_string(),
            llvm_name: "le_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Poly("'_a".to_string()), TypeName::Poly("'_a".to_string()), TypeName::Bool),
        }),
        ("(>)".to_string(), StdFun {
            name: "(>)".to_string(),
            llvm_name: "g_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Poly("'_a".to_string()), TypeName::Poly("'_a".to_string()), TypeName::Bool),
        }),
        ("(>=)".to_string(), StdFun {
            name: "(>=)".to_string(),
            llvm_name: "ge_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Poly("'_a".to_string()), TypeName::Poly("'_a".to_string()), TypeName::Bool),
        }),
        ("(=)".to_string(), StdFun {
            name: "(=)".to_string(),
            llvm_name: "eq_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Poly("'_a".to_string()), TypeName::Poly("'_a".to_string()), TypeName::Bool),
        }),
        ("(==)".to_string(), StdFun {
            name: "(==)".to_string(),
            llvm_name: "peq_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Poly("'_a".to_string()), TypeName::Poly("'_a".to_string()), TypeName::Bool),
        }),
        ("(<>)".to_string(), StdFun {
            name: "(<>)".to_string(),
            llvm_name: "neq_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Poly("'_a".to_string()), TypeName::Poly("'_a".to_string()), TypeName::Bool),
        }),
        ("(!=)".to_string(), StdFun {
            name: "(!=)".to_string(),
            llvm_name: "pneq_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Poly("'_a".to_string()), TypeName::Poly("'_a".to_string()), TypeName::Bool),
        }),
        ("print_int".to_string(), StdFun {
            name: "print_int".to_string(),
            llvm_name: "print_int".to_string(),
            fun_type: RT_TYPE,
            tp: TypeName::Function(Box::new(TypeName::Int), Box::new(TypeName::Unit)),
        }),
        ("(&&)".to_string(), StdFun {
            name: "&&".to_string(),
            llvm_name: "land_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Bool, TypeName::Bool, TypeName::Bool),
        }),
        ("(||)".to_string(), StdFun {
            name: "||".to_string(),
            llvm_name: "lor_r2".to_string(),
            fun_type: RT_TYPE,
            tp: bin_op(TypeName::Bool, TypeName::Bool, TypeName::Bool),
        }),
        ("get_field".to_string(), StdFun {
            name: "get_field".to_string(),
            llvm_name: "rt_get_field".to_string(),
            fun_type: FunType(IsSystem::SystemFun, IsVararg::NotVararg),
            tp: TypeName::Function(
                Box::new(TypeName::Poly("'_a".to_string())),
                Box::new(TypeName::Function(
                    Box::new(TypeName::Int),
                    Box::new(TypeName::Poly("'_b".to_string())),
                )),
            ),
        }),
        ("_create_tuple".to_string(), StdFun {
            name: "_create_tuple".to_string(),
            llvm_name: "rt_create_tuple".to_string(),
            fun_type: FunType(IsSystem::SystemFun, IsVararg::Vararg),
            tp: TypeName::Function(Box::new(TypeName::Int), Box::new(TypeName::Poly("_vargs".to_string()))),
        }),
        ("_create_closure".to_string(), StdFun {
            name: "_create_closure".to_string(),
            llvm_name: "rt_create_closure".to_string(),
            fun_type: FunType(IsSystem::SystemFun, IsVararg::NotVararg),
            tp: TypeName::Function(
                Box::new(TypeName::Poly("'_a".to_string())),
                Box::new(TypeName::Poly("'_b".to_string())),
            ),
        }),
        ("_args_application".to_string(), StdFun {
            name: "_args_application".to_string(),
            llvm_name: "rt_application".to_string(),
            fun_type: FunType(IsSystem::SystemFun, IsVararg::Vararg),
            tp: TypeName::Function(
                Box::new(TypeName::Poly("_closure".to_string())),
                Box::new(TypeName::Function(
                    Box::new(TypeName::Int),
                    Box::new(TypeName::Poly("_varargs".to_string())),
                )),
            ),
        }),
        ("_globals_feat".to_string(), StdFun {
            name: "_globals_feat".to_string(),
            llvm_name: "rt_globals".to_string(),
            fun_type: FunType(IsSystem::SystemFun, IsVararg::Vararg),
            tp: TypeName::Function(
                Box::new(TypeName::Poly("_globs_count".to_string())),
                Box::new(TypeName::Poly("_varargs".to_string())),
            ),
        }),
    ])
}

#[derive(Debug)]
#[derive(Clone)]
pub struct State<'ctx> {
    current_function: String,
    prev_state: Option<Box<State<'ctx>>>,
    local_variables: HashMap<LLVMName, BasicValueEnum<'ctx>>,
}

impl<'ctx> State<'ctx> {
    pub fn new(current_function: String, prev_state: Option<Box<State<'ctx>>>) -> Self {
        State {
            current_function,
            prev_state,
            local_variables: HashMap::new(),
        }
    }
}

pub struct LLVMCodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    global_functions: HashMap<LLVMName, (FunctionType<'ctx>, FunctionValue<'ctx>)>,
    state: Box<State<'ctx>>,
    i64_type: IntType<'ctx>,
    llvm_suffix: String,
    tail_call_enforce: bool,
}

impl<'ctx> LLVMCodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str, tail_call_enforce: bool) -> Self {
        let module = context.create_module(module_name);
        let i64_type = context.i64_type();

        // Set target triple for RISC-V
        module.set_triple(&inkwell::targets::TargetTriple::create("riscv64-unknown-linux-gnu"));

        let state = State::new("".to_string(), None);

        LLVMCodeGen {
            context,
            module,
            tail_call_enforce,
            global_functions: HashMap::new(),
            state: Box::new(state),
            i64_type,
            llvm_suffix: "_llvm".to_string(),
        }
    }

    fn enter_new_scope(&mut self, name: &str) {
        let prev_state = Some(self.state.clone());
        self.state = Box::from(State::new(name.to_string(), prev_state));
    }

    fn define_global(&mut self, name: &str) -> GlobalValue<'ctx> {
        let global_name = LLVMName::from(get_global_name(LLVMName::from(name).get_name().as_str())).get_full_name();
        let global = self.module.add_global(self.i64_type, None, global_name.as_str());
        global.set_initializer(&self.i64_type.const_int(0, false));
        global
    }

    fn exit_current_scope(&mut self) {
        if let Some(prev_state) = self.state.prev_state.take() {
            self.state = prev_state;
        } else {
            panic!("No previous state to exit");
        }
    }

    pub fn nat2ml(&self, nat: IntValue<'ctx>, builder: &mut Builder<'ctx>) -> IntValue<'ctx> {
        let shift = self.i64_type.const_int(1, false);
        let shl_result = builder.build_left_shift(nat, shift, "").unwrap();
        let one = self.i64_type.const_int(1, false);
        builder.build_int_add(shl_result, one, "").unwrap()
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn count_args(&self, tp: &TypeName) -> usize {
        match tp {
            TypeName::Function(_, ret_tp) => 1 + self.count_args(ret_tp),
            _ => 0,
        }
    }

    pub fn gen_function_type_std(&self, tp: &TypeName, vararg_flag: IsVararg) -> FunctionType<'ctx> {
        let args_cnt = self.count_args(tp);
        let mut args = Vec::with_capacity(args_cnt);
        for _ in 0..args_cnt {
            args.push(BasicMetadataTypeEnum::from(self.i64_type));
        }

        match vararg_flag {
            IsVararg::NotVararg => self.i64_type.fn_type(args.as_slice(), false),
            IsVararg::Vararg => self.i64_type.fn_type(&args, true),
        }
    }

    pub fn build_mlrt_create_closure(&'_ self, target_fun: BasicValueEnum<'ctx>, args_count: i32, builder: &mut Builder<'ctx>) -> CallSiteValue<'_> {
        let int_ptr = builder.build_ptr_to_int(target_fun.into_pointer_value(), self.i64_type, "ptrtoint").unwrap();
        let args = [
            BasicMetadataValueEnum::from(int_ptr),
            BasicMetadataValueEnum::from(self.i64_type.const_int(args_count as u64, false))
        ];


        let closure_func = self.global_functions.get(&LLVMName::from("_create_closure".to_string()))
            .map(|(_, func)| *func)
            .expect("rt_create_closure function not found");

        // Build the call
        let call_result = builder.build_call(closure_func, &args, "").unwrap();
        if self.tail_call_enforce {
            call_result.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        };

        call_result
    }


    pub fn declare_std_fun(&mut self, std_fun: &StdFun) -> FunctionValue<'ctx> {
        let fun_tp = self.gen_function_type_std(&std_fun.tp, std_fun.fun_type.1);
        let fun_val = self.module.add_function(&std_fun.llvm_name, fun_tp, None);

        self.global_functions.insert(
            LLVMName::from(std_fun.name.clone()),
            (fun_tp, fun_val),
        );

        fun_val
    }

    pub fn declare_function(&mut self, name: &str, args_cnt: usize) -> FunctionValue<'ctx> {
        let mut args = Vec::with_capacity(args_cnt);
        for _ in 0..args_cnt {
            args.push(BasicMetadataTypeEnum::from(self.i64_type));
        }

        let fun_tp = self.i64_type.fn_type(&args, false);
        let fun_val = self.module.add_function(name, fun_tp, None);

        self.global_functions.insert(
            LLVMName::from(name),
            (fun_tp, fun_val),
        );
        fun_val
    }

    pub fn build_ret_mlvoid(&self) -> IntValue<'ctx> {
        self.i64_type.const_int(0, false)
    }

    fn localise_var(&mut self, name: &str, glob_var: GlobalValue<'ctx>) -> BasicValueEnum<'ctx> {
        let current_fun = self.state.current_function.clone();
        let name_to_find = LLVMName::from(current_fun);
        let fun_val = self.global_functions.get(&name_to_find).unwrap().1;
        let eb = fun_val.get_first_basic_block().unwrap();

        // Create a new builder and position it at the beginning of the entry block
        let builder = self.context.create_builder();
        builder.position_at_end(eb);

        // Create a load instruction for the global variable
        let loc_var = builder.build_load(self.i64_type, glob_var.as_pointer_value(), glob_var.get_name().to_str().unwrap()).unwrap();

        // Add the local variable to state - store the pointer value, not the loaded value
        self.state.local_variables.insert(LLVMName::from(name), loc_var);

        loc_var
    }


    pub fn build_nat2ml_icmp(&self, cmode: IntPredicate, val1: IntValue<'ctx>, val2: IntValue<'ctx>, builder: &mut Builder<'ctx>) -> IntValue<'ctx> {
        let bool1 = builder.build_int_compare(cmode, val1, val2, "").unwrap();
        let bool64 = builder.build_int_z_extend(bool1, self.i64_type, "").unwrap();
        self.nat2ml(bool64, builder)
    }

    pub fn build_identifier(&mut self, idname: &str, _: &mut Builder<'ctx>) -> BasicValueEnum<'ctx> {
        //println!("Building identifier: {}", idname);
        let name = LLVMName::from(idname);
        //println!("Finding {:?} in {:?}", name, self.state.local_variables);
        if let Some(local_ptr) = self.state.local_variables.get(&name) {
            *local_ptr
        } else {
            let name = LLVMName::from(get_global_name(LLVMName::from(idname).get_name().as_str()));
            //println!("{:?} : {:?}", name, self.module.get_globals().into_iter());
            let global_val = self.module.get_global(name.get_full_name().as_str()).unwrap();

            self.localise_var(name.get_full_name().as_str(), global_val)
        }
    }

    pub fn build_imm_expr(&mut self, imm: &ImmExpr, builder: &mut Builder<'ctx>) -> BasicValueEnum<'ctx> {
        match imm {
            Bool(b) => {
                let nat_val = if *b { 1 } else { 0 };
                self.nat2ml(self.i64_type.const_int(nat_val, false), builder).as_basic_value_enum()
            }
            Int(i) => self.nat2ml(self.i64_type.const_int(*i as u64, false), builder).as_basic_value_enum(),
            Unit | Nil => self.nat2ml(self.i64_type.const_int(0, false), builder).as_basic_value_enum(),
            Tuple(_) => {
                // Placeholder
                self.i64_type.const_int(0, false).as_basic_value_enum()
            }
            Identifier(id_name) => {
                self.build_identifier(id_name, builder)
            }
        }
    }

    fn build_mlrt_apply_args_to_closure(&self, closure: BasicValueEnum<'ctx>, applied_args: Vec<BasicValueEnum<'ctx>>, builder: &mut Builder<'ctx>) -> CallSiteValue<'ctx> {
        let applied_args_cnt = applied_args.len();
        let mut args = Vec::with_capacity(applied_args_cnt + 2);

        // Add closure as first argument
        args.push(BasicMetadataValueEnum::from(closure));

        // Add argument count as second argument
        args.push(BasicMetadataValueEnum::from(self.i64_type.const_int(applied_args_cnt as u64, false)));

        // Add all applied arguments
        for arg in applied_args {
            args.push(BasicMetadataValueEnum::from(arg));
        }

        let apply_func = self.global_functions.get(&LLVMName::from("_args_application".to_string()))
            .map(|(_, func)| *func)
            .expect("mlrt_apply_args_to_closure function not found");

        // Build the call
        builder.build_call(apply_func, &args, "").unwrap()
    }

    fn build_apply_closure(&self, builder: &mut Builder<'ctx>, closure: BasicValueEnum<'ctx>, args_val: Vec<BasicValueEnum<'ctx>>) -> CallSiteValue<'ctx> {
        self.build_mlrt_apply_args_to_closure(closure, args_val, builder)
    }

    pub fn build_cexpr(&mut self, cexp: &CExpr, builder: &mut Builder<'ctx>) -> IntValue<'ctx> {
        match cexp {
            CExpr::ImmExpr(imm) => IntValue::try_from(self.build_imm_expr(imm, builder)).unwrap(),
            CExpr::Application(fun_c, imm_arg_hd, imm_arg_tl) => {
                // Build arguments list
                let mut args_val = Vec::new();
                args_val.push(self.build_imm_expr(imm_arg_hd, builder));
                for arg in imm_arg_tl {
                    args_val.push(self.build_imm_expr(arg, builder));
                }

                // Handle special cases for logical and comparison operators
                if let ImmExpr::Identifier(fun_name) = &**fun_c {
                    match (fun_name.as_str(), &args_val[..]) {
                        ("&&", [a, b]) => {
                            return builder.build_and(
                                IntValue::try_from(*a).unwrap(),
                                IntValue::try_from(*b).unwrap(),
                                "",
                            ).unwrap();
                        }

                        ("||", [a, b]) => {
                            return builder.build_or(
                                IntValue::try_from(*a).unwrap(),
                                IntValue::try_from(*b).unwrap(),
                                "",
                            ).unwrap();
                        }
                        ("==", [a, b]) => {
                            return self.build_nat2ml_icmp(
                                IntPredicate::EQ,
                                IntValue::try_from(*a).unwrap(),
                                IntValue::try_from(*b).unwrap(),
                                builder,
                            );
                        }
                        ("!=", [a, b]) => {
                            return self.build_nat2ml_icmp(
                                IntPredicate::NE,
                                IntValue::try_from(*a).unwrap(),
                                IntValue::try_from(*b).unwrap(),
                                builder,
                            );
                        }
                        _ => ()
                    }
                }

                // Try to find a matching function in global functions
                let mut fun_name = LLVMName::from("");
                let fun_val = if let ImmExpr::Identifier(name) = &**fun_c {
                    fun_name = LLVMName::from(name.clone());
                    // Check if it's a standard function
                    if let Some(std_fun) = get_std_funs().get(name) {
                        // Look up the function in our global functions map
                        if let Some((_, func_val)) = self.global_functions.get(&LLVMName::from(std_fun.name.clone())) {
                            Some(*func_val)
                        } else {
                            None
                        }
                    } else {
                        // Look up in global functions
                        if let Some((_, func_val)) = self.global_functions.get(&LLVMName::from(name.clone())) {
                            Some(*func_val)
                        } else {
                            None
                        }
                    }
                } else {
                    // For non-identifier functions, we'll treat them as closures
                    None
                };

                // If we found a function with matching number of arguments, call it directly
                if let Some(func_val) = fun_val {
                    if let Some((fun_type, _)) = self.global_functions.get(&fun_name) {
                        let expected_args = fun_type.count_param_types();
                        if expected_args == args_val.len() as u32 {
                            // Build direct function call
                            let args: Vec<_> = args_val.iter().map(|v| BasicMetadataValueEnum::from(*v)).collect();
                            let call_val = builder.build_call(func_val, &args, "").unwrap();
                            if self.tail_call_enforce {
                                call_val.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
                            }
                            let res = call_val.as_any_value_enum().into_int_value();

                            return res;
                        }
                    }
                }

                // For general function calls or closure applications
                // Get the function value
                let fun_c_val = self.build_imm_expr(fun_c, builder);
                self.build_apply_closure(builder, fun_c_val, args_val).try_as_basic_value().basic().unwrap().into_int_value()
            }
            CExpr::IfThenElse { condition, then_branch, else_branch } => {
                // Build the condition
                let flag = self.build_imm_expr(condition, builder).into_int_value();


                // Convert flag to boolean (assuming 0 means false, anything else means true)
                let bool1 = builder.build_int_truncate(
                    builder.build_right_shift(flag, self.i64_type.const_int(1, false), true, "").unwrap(), self.context.bool_type(),
                    "",
                ).unwrap();

                // Get current function information
                let current_function = self.state.current_function.clone();
                let fun_val = self.global_functions.get(&LLVMName::from(current_function.clone())).unwrap().1;

                // Create the continuation block
                let continue_block = self.context.append_basic_block(fun_val, "continue");

                // Helper to create branch blocks
                let mut create_branch = |branch_body: &AExpr, fun_val: FunctionValue<'ctx>| -> (IntValue<'ctx>, BasicBlock<'ctx>) {
                    let branch_block = self.context.append_basic_block(fun_val, "");
                    let mut branch_builder = self.context.create_builder();
                    branch_builder.position_at_end(branch_block);

                    let result = self.build_aexpr(branch_body, &mut branch_builder);
                    let last_inst = branch_builder.build_unconditional_branch(continue_block).unwrap();
                    let block = branch_builder.get_insert_block().unwrap();
                    branch_builder.position_at(block, &last_inst);

                    (result, block)
                };

                // Create then and else branches
                let (then_result, then_block) = create_branch(then_branch, fun_val);
                let (else_result, else_block) = create_branch(else_branch, fun_val);
                // Create conditional branch
                builder.build_conditional_branch(bool1, then_block, else_block).unwrap();

                // Position builder at continuation block
                builder.position_at_end(continue_block);

                // Create phi node
                let phi_node = builder.build_phi(self.i64_type, "").unwrap();
                phi_node.add_incoming(&[(&then_result, then_block), (&else_result, else_block)]);
                phi_node.as_basic_value().into_int_value()
            }
        }
    }

    pub fn build_aexpr(&mut self, aexpr: &AExpr, builder: &mut Builder<'ctx>) -> IntValue<'ctx> {
        match aexpr {
            AExpr::CExpr(cexp) => self.build_cexpr(cexp, builder),
            AExpr::LetIn { name, body, in_body } => {
                let body_val = self.build_cexpr(body, builder);
                //println!("Building let-in expression: {}", name);
                self.state.local_variables.insert(LLVMName::from(name.clone()), BasicValueEnum::from(body_val));
                //println!("Inserted: {:?}", (LLVMName::from(name.clone()), BasicValueEnum::from(body_val)));

                let in_body_val = self.build_aexpr(in_body, builder);
                in_body_val
            }
        }
    }

    pub fn build_anf_decl(&mut self, anf_decl: &AnfDecl, builder: &mut Builder<'ctx>) -> Result<(), String> {
        match anf_decl {
            AnfDecl::SingleLet { is_rec: _, single_anf_binding } => {
                match single_anf_binding {
                    SingleAnfBinding::Let { name, args, .. } => {
                        // Process function definition
                        let fun_name = name.clone();
                        let args_count = args.len();

                        // Create function
                        let _ = self.declare_function(&fun_name, args_count);
                        self.define_global(&fun_name);

                        self.define_single_anf_bind(single_anf_binding, builder)?;

                        Ok(())
                    }
                }
            }
        }
    }

    // Implementation of the requested function
    pub fn declare_and_define_rt_globs(&mut self, builder: &mut Builder<'ctx>) -> Result<(), String> {
        let mut help = |std_fun: &StdFun| -> Result<(), String> {
            match std_fun.fun_type.0 {
                IsSystem::SystemFun => {
                    let name = LLVMName::from(get_global_name(LLVMName::from(std_fun.name.clone()).get_name().as_str())).get_full_name();
                    let glob_c = match self.module.get_global(name.as_str()) {
                        Some(g_val) => { g_val }
                        None => self.define_global(&std_fun.name)
                    };
                    let target_fun = self.global_functions.get(&LLVMName::from(std_fun.name.clone()))
                        .ok_or_else(|| format!("Function {} not found", std_fun.name))?
                        .1;

                    let clos = self.build_mlrt_create_closure(
                        target_fun.as_global_value().as_basic_value_enum(),
                        self.count_args(&std_fun.tp) as i32,
                        builder,
                    );

                    builder.build_store(glob_c.as_pointer_value(), clos.try_as_basic_value().basic().unwrap().into_int_value()).unwrap();
                    Ok(())
                }
            }
        };

        for (_, std_fun) in get_std_funs().iter() {
            help(std_fun)?;
        }

        Ok(())
    }

    pub fn define_single_anf_bind(&mut self, single_anf_binding: &SingleAnfBinding, builder: &mut Builder<'ctx>) -> Result<(), String> {
        // Extract information from the binding
        let (name, args_name, body) = match single_anf_binding {
            SingleAnfBinding::Let { name, args, body } => (name, args, body),
        };

        // Find the function in global functions (it should already exist from declare_function)
        let fun_name = LLVMName::from(name.clone());
        let (_, fun_val) = *self.global_functions.get(&fun_name)
            .ok_or_else(|| format!("Function {} not found", name))?;

        // Create entry block for the function
        let entry_b = self.context.append_basic_block(fun_val, "entry");
        let mut new_builder = self.context.create_builder();


        new_builder.position_at_end(entry_b);
        self.enter_new_scope(name);

        // Get function parameters
        let args_val = fun_val.get_params();
        let name_and_val: Vec<(String, BasicValueEnum)> = args_name.iter()
            .zip(args_val.iter())
            .map(|(nm, llval)| (nm.clone(), *llval))
            .collect();

        // Add local variables for parameters
        for (nm, llval) in name_and_val {
            self.state.local_variables.insert(LLVMName::from(nm), llval);
        }

        // Build the function body
        let result = self.build_aexpr(body, &mut new_builder);

        // Return the result
        new_builder.build_return(Some(&result)).unwrap();

        self.exit_current_scope();
        // Define helper variable (closure)
        let helper_name = LLVMName::from(get_global_name(name));
        let args_cnt = args_name.len();

        let helper_val = if args_cnt == 0 {
            let args = [];
            let res = builder.build_call(fun_val, &args, &helper_name.get_full_name()).unwrap();
            if self.tail_call_enforce {
                res.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
            };

            res
        } else {
            // With arguments case - create empty closure
            let res = self.build_mlrt_create_closure(fun_val.as_global_value().as_basic_value_enum(), args_cnt as i32, builder);

            res
        };

        // Find or create the global helper pointer
        let helper_ptr = self.module.get_global(helper_name.get_full_name().as_str())
            .unwrap();

        // Store the helper value
        builder.build_store(helper_ptr.as_pointer_value(), helper_val.try_as_basic_value().basic().unwrap()).unwrap();

        Ok(())
    }

    pub fn create_init(&mut self) -> Result<FunctionValue<'ctx>, String> {
        let init_name = format!("init{}", self.llvm_suffix);
        let func = self.declare_function(&init_name, 0);

        // Create entry block
        let entry_block = self.context.append_basic_block(func, "entry");
        let mut builder = self.context.create_builder();
        builder.position_at_end(entry_block);

        self.enter_new_scope(&init_name);

        for (_, fun) in get_std_funs().iter() {
            let _ = self.define_global(fun.name.as_str());
        }

        // Collect all global variables
        let mut globs = Vec::new();

        // Iterate through all globals in the module
        for global in self.module.get_globals() {
            // Convert global to pointer value and then to integer
            let ptr_val = global.as_pointer_value();
            let int_val = builder.build_ptr_to_int(ptr_val, self.i64_type, "").unwrap();
            globs.push(int_val);
        }

        // Prepend the count of globals
        let count = self.i64_type.const_int(globs.len() as u64, false);
        globs.insert(0, count);

        // Prepare arguments for function call
        let args: Vec<_> = globs.iter().map(|&val| BasicMetadataValueEnum::from(val)).collect();

        // Get the function value for mlrt_globals_feat
        let func_name = LLVMName::from("_globals_feat".to_string());
        let func_val = self.global_functions.get(&func_name)
            .expect("mlrt_globals_feat function not found")
            .1;

        // Build the call
        let _ = builder.build_call(func_val, &args, "").unwrap();
        self.declare_and_define_rt_globs(&mut builder).expect("cannot define global ptrs");


        // Add return statement
        let ret_val = self.build_ret_mlvoid();
        let _ = builder.build_return(Some(&ret_val));
        self.exit_current_scope();

        Ok(func)
    }

    pub fn create_main(&mut self, init_func: FunctionValue<'ctx>, anf_decl_lst: &[AnfDecl]) -> Result<(), String> {
        let main_name = "main".to_string();
        let main_func = self.declare_function(&main_name, 0);

        let entry_block = self.context.append_basic_block(main_func, "entry");
        let mut builder = self.context.create_builder();
        builder.position_at_end(entry_block);
        self.enter_new_scope(&main_name);

        let args = [];
        let _ = builder.build_call(init_func, &args, "");

        for decl in anf_decl_lst {
            self.build_anf_decl(decl, &mut builder)?;
        }

        let ret_val = self.build_ret_mlvoid();
        let _ = builder.build_return(Some(&ret_val));

        self.exit_current_scope();

        Ok(())
    }

    pub fn start_compile(&mut self, prog: &AnfProg) -> Result<(), String> {
        // Declare all standard functions
        for (_, std_fun) in get_std_funs().iter() {
            self.declare_std_fun(&std_fun.clone());
        }

        // Create init function
        let init_func = self.create_init()?;

        // Create main function
        self.create_main(init_func, &prog.0)?;


        Ok(())
    }

    pub fn print_module(&self, filename: &str) {
        let opt = PassBuilderOptions::create();

        Target::initialize_riscv(&InitializationConfig::default());
        let m_opt = TargetMachineOptions::new();
        let triple = inkwell::targets::TargetTriple::create("riscv64-unknown-linux-gnu");
        let target = Target::from_triple(&triple).unwrap();
        let machine = target.create_target_machine_from_options(&triple, m_opt).unwrap();
        if self.tail_call_enforce {
            let _ = self.module.run_passes("tailcallelim", &(machine), opt);
        };

        self.module.print_to_file(filename).unwrap();
    }
}

pub fn compile(out_name: &str, prog: &AnfProg, enforce_tailcall: bool) -> Result<(), String> {
    let context = Context::create();
    let mut codegen = LLVMCodeGen::new(&context, "Rust2", enforce_tailcall);

    codegen.start_compile(prog)?;

    codegen.print_module(out_name);

    Ok(())
}
