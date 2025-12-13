use std::env;
use std::fs;

use axis_core::parse::AxisCoreParser;
use axis_core::parse_ast::parse_program;
use axis_core::desugar::desugar_program;
use axis_core::eval::eval;
use axis_core::value::Env;
use pest::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let input = fs::read_to_string(filename)
        .expect("failed to read file");

    // 1. Parse
    let pairs = AxisCoreParser::parse(
        axis_core::parse::Rule::program,
        &input,
    ).expect("parse error");

    let ast = parse_program(pairs);

    // 2. Desugar
    let core_fns = desugar_program(&ast)
        .expect("desugar error");

    // For now: assume one function, called `main`
    let (_name, core_expr) = &core_fns[0];

    // 3. Evaluate
    let env = Env::new();
    let value = eval(core_expr, &env)
        .expect("eval error");

    println!("Result: {:?}", value);
}

