mod parse;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: axis <file.axis>");
        std::process::exit(1);
    }

    let path = &args[1];
    let src = fs::read_to_string(path)
        .unwrap_or_else(|e| {
            eprintln!("failed to read {}: {}", path, e);
            std::process::exit(1);
        });

    match parse::parse_program(&src) {
        Ok(_) => {
            println!("✓ parsed {}", path);
        }
        Err(e) => {
            eprintln!("✗ parse error in {}:\n{}", path, e);
            std::process::exit(1);
        }
    }
}
