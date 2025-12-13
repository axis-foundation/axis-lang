use pest::Parser;

use axis_core::parse::AxisCoreParser;
use axis_core::parse::Rule;
use axis_core::parse_ast::parse_program;

use axis_core::desugar::desugar_block;
use axis_core::eval::eval;
use axis_core::value::{Env, Value};

use axis_core::surface_eval::{
    env_from_core,
    eval_block as eval_surface_block,
    value_from_svalue,
};

#[test]
fn desugar_preserves_nested_block_expression() {
    let src = r#"
fn main(x: Int) -> Int {
    let t: (Int, Int) = {
        let a: Int = x;
        let b: Int = 4;
        (a, b)
    };
    t.2
}
"#;

    // Parse → AST
    let pairs = AxisCoreParser::parse(Rule::program, src).expect("parse failed");
    let prog = parse_program(pairs);
    let f = &prog.functions[0];

    // Input environment: x = 9
    let mut env_core = Env::new();
    env_core.insert("x".to_string(), Value::Int(9));

    // Surface eval
    let env_surf = env_from_core(&env_core).expect("env conversion failed");
    let surf_v = eval_surface_block(&f.body, &env_surf).expect("surface eval failed");
    let surf_v = value_from_svalue(&surf_v).expect("surface non-ground result");

    // Core eval
    let core_expr = desugar_block(&f.body).expect("desugar failed");
    let core_v = eval(&core_expr, &env_core).expect("core eval failed");

    assert_eq!(surf_v, core_v);
}
