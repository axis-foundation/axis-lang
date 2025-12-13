use pest::Parser;

use axis_core::parse::AxisCoreParser;
use axis_core::parse::Rule;
use axis_core::parse_ast::parse_program;
use axis_core::desugar::desugar_program;
use axis_core::eval::eval;
use axis_core::value::{Env, Value};

#[test]
fn eval_simple_let() {
    let src = r#"
fn main(x: Int) -> Int {
    let y: Int = 2;
    let z: Int = y;
    z
}
"#;

    // 1. Parse
    let pairs = AxisCoreParser::parse(Rule::program, src)
        .expect("parse failed");

    let ast = parse_program(pairs);

    // 2. Desugar
    let core_fns = desugar_program(&ast)
        .expect("desugar failed");

    // Expect exactly one function
    let (_name, core_expr) = &core_fns[0];

    // 3. Evaluate
    let env = Env::new();
    let result = eval(core_expr, &env)
        .expect("evaluation failed");

    // 4. Assert
    assert_eq!(result, Value::Int(2));
}
