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
        "rust-run" => {
            if let Err(msg) = rust_run(&args[2]) {
                eprintln!("{}", msg);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn rust_run(filename: &str) -> Result<(), String> {
    // ------------------------------------------------------------
    // Read Axis source
    // ------------------------------------------------------------
    let input = fs::read_to_string(filename)
        .map_err(|e| format!("failed to read {}: {}", filename, e))?;

    // ------------------------------------------------------------
    // Parse → AST
    // ------------------------------------------------------------
    let pairs = AxisCoreParser::parse(Rule::program, &input)
        .map_err(|e| format!("parse failed: {}", e))?;

    let program = parse_program(pairs);

    if program.functions.is_empty() {
        return Err("no functions found".to_string());
    }

    // For now: assume first function is entry point
    let func = &program.functions[0];

    // ------------------------------------------------------------
    // Desugar
    // ------------------------------------------------------------
    let core_expr = desugar_block(&func.body)
        .map_err(|e| format!("desugar failed: {:?}", e))?;

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
        .map_err(|e| format!("failed to create temp directory: {}", e))?;

    let mut rs_path = PathBuf::from(&tmp_dir);
    rs_path.push("out.rs");

    let mut bin_path = PathBuf::from(&tmp_dir);
    bin_path.push("out_bin");

    fs::write(&rs_path, rust_src)
        .map_err(|e| format!("failed to write generated Rust file: {}", e))?;

    // ------------------------------------------------------------
    // Compile with rustc
    // ------------------------------------------------------------
    let status = Command::new("rustc")
        .arg(&rs_path)
        .arg("-O")
        .arg("-o")
        .arg(&bin_path)
        .status()
        .map_err(|e| format!("failed to invoke rustc: {}", e))?;

    if !status.success() {
        return Err("rustc compilation failed".to_string());
    }

    // ------------------------------------------------------------
    // Run generated binary
    // ------------------------------------------------------------
    let output = Command::new(&bin_path)
        .output()
        .map_err(|e| format!("failed to run generated binary: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("generated program failed\n{}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout.trim());

    Ok(())
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

