use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use pest::Parser;

use axis_core::parse::AxisCoreParser;
use axis_core::parse::Rule;
use axis_core::parse_ast::parse_program;
use axis_core::desugar::desugar_block;
use axis_core::codegen_rust::emit_rust_program;
use axis_core::ast::Type;
use axis_core::core_ast::{CoreAtom, CoreAtomBase, CoreExpr};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("usage:");
        eprintln!("  axis-cli rust-run <file.axis>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "rust-run" => rust_run(&args[2]),
        _ => {
            eprintln!("unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn rust_run(filename: &str) {
    // ------------------------------------------------------------
    // Read Axis source
    // ------------------------------------------------------------
    let input = fs::read_to_string(filename)
        .expect("failed to read .axis file");

    // ------------------------------------------------------------
    // Parse → AST
    // ------------------------------------------------------------
    let pairs = AxisCoreParser::parse(Rule::program, &input)
        .expect("parse failed");

    let program = parse_program(pairs);

    if program.functions.is_empty() {
        panic!("no functions found");
    }

    // For now: assume first function is entry point
    let func = &program.functions[0];

    // ------------------------------------------------------------
    // Desugar
    // ------------------------------------------------------------
    let core_expr = desugar_block(&func.body)
        .expect("desugar failed");

    // Bind the entry function parameter to a default value so programs that
    // reference it (e.g. `x`) don't panic with "unbound var".
    let default_arg = default_value_expr(&func.param.ty);
    let core_expr = CoreExpr::LetIn {
        name: func.param.name.clone(),
        ty: func.param.ty.clone(),
        value: Box::new(default_arg),
        body: Box::new(core_expr),
    };

    // ------------------------------------------------------------
    // Emit Rust
    // ------------------------------------------------------------
    let rust_src = emit_rust_program(&core_expr);

    // ------------------------------------------------------------
    // Prepare temp directory
    // ------------------------------------------------------------
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push("axis_rust_run");

    fs::create_dir_all(&tmp_dir)
        .expect("failed to create temp directory");

    let mut rs_path = PathBuf::from(&tmp_dir);
    rs_path.push("out.rs");

    let mut bin_path = PathBuf::from(&tmp_dir);
    bin_path.push("out_bin");

    fs::write(&rs_path, rust_src)
        .expect("failed to write generated Rust file");

    // ------------------------------------------------------------
    // Compile with rustc
    // ------------------------------------------------------------
    let status = Command::new("rustc")
        .arg(&rs_path)
        .arg("-O")
        .arg("-o")
        .arg(&bin_path)
        .status()
        .expect("failed to invoke rustc");

    if !status.success() {
        panic!("rustc compilation failed");
    }

    // ------------------------------------------------------------
    // Run generated binary
    // ------------------------------------------------------------
    let output = Command::new(&bin_path)
        .output()
        .expect("failed to run generated binary");

    if !output.status.success() {
        panic!("generated program failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout.trim());
}

fn default_value_expr(ty: &Type) -> CoreExpr {
    match ty {
        Type::Int => CoreExpr::Atom(CoreAtom {
            base: CoreAtomBase::Lit(axis_core::ast::Lit::Int(0)),
            projs: vec![],
        }),
        Type::Bool => CoreExpr::Atom(CoreAtom {
            base: CoreAtomBase::Lit(axis_core::ast::Lit::Bool(false)),
            projs: vec![],
        }),
        Type::Unit => CoreExpr::Atom(CoreAtom {
            base: CoreAtomBase::Lit(axis_core::ast::Lit::Unit),
            projs: vec![],
        }),
        Type::Tuple(elems) => {
            let xs = elems.iter().map(default_value_expr).collect::<Vec<_>>();
            CoreExpr::Atom(CoreAtom {
                base: CoreAtomBase::Tuple(xs),
                projs: vec![],
            })
        }
        Type::Func(_, _) => panic!("rust-run: cannot synthesize default value for function type"),
    }
}

