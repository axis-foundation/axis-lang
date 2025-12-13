use pest::Parser;

use axis_core::parse::AxisCoreParser;
use axis_core::parse::Rule;
use axis_core::parse_ast::parse_program;

#[test]
fn parses_semantic_battery_example() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let path = std::path::Path::new(&manifest_dir)
        .join("../examples/semantic_battery.axis");

    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));

    let pairs = AxisCoreParser::parse(Rule::program, &src)
        .unwrap_or_else(|e| panic!("parse failed: {}", e));

    let prog = parse_program(pairs);
    assert!(!prog.functions.is_empty());
}
