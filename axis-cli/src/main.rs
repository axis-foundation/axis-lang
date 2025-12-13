use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: axis <file.axis>");
        std::process::exit(1);
    }

    let src = fs::read_to_string(&args[1]).expect("failed to read file");

    match axis_core::parse::parse_program(&src) {
        Ok(_) => println!("✓ parsed {}", args[1]),
        Err(e) => {
            eprintln!("✗ parse error:\n{}", e);
            std::process::exit(1);
        }
    }
}
