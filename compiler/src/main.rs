use crate::anf::ANFConverter;
use crate::ast::Program;
use clap::Parser;
use std::fs;

mod parser;
mod lexer;
mod ast;
mod anf;
mod ll;
mod cc;
mod codegen;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enabling the generation of LLVM IR tail call optimization
    #[arg(short = 't', long = "enforce_tail_call", default_value_t = false)]
    enforce_tail_call: bool,
    /// Code source
    #[arg(short = 's', long = "source")]
    source: String,
    /// Code out
    #[arg(short = 'o', long = "out")]
    out: String,
}


fn main() {
    let args = Args::parse();
    let source = fs::read_to_string(args.source).expect("Unable to read file");
    let out = args.out;
    let mut parser = parser::Parser::new(source.as_str());
    let anf_converter = ANFConverter::new();
    anf_converter.init();
    let program = parser.parse_program();
    let cc = cc::ClConverter::new();
    let converted_cc = cc.cc_ast(&program.items);
    //println!("{:#?}", converted_cc);
    let lifted_ll = ll::lift_ast(&Program { items: converted_cc });
    // println!("{:?}", lifted_ll);
    let mut ll_ref = String::new();
    crate::ll::ll_pp::pp_llprogram(&mut ll_ref, &lifted_ll).unwrap();
    //println!("{:#?}", ll_ref);

    let anfed_ast = anf_converter.transform_anf(lifted_ll);
    //println!("{}", anf::anf_pp::program_to_string(&anfed_ast.0));

    let _ = codegen::compile(out.as_str(), &anfed_ast, args.enforce_tail_call);
    //println!("{:?}", final_code);
}