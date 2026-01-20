use std::env;
use std::fs;
use std::process;
use b::lexer::scanner::Scanner;

fn main() {
    // 1. Collect command line arguments
    let args: Vec<String> = env::args().collect();

    // 2. Check if a filename was provided
    // args[0] is the program name, args[1] is the first argument
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin lex <filename.b>");
        process::exit(1);
    }

    let filename = &args[1];

    // 3. Read the file content
    let content = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading file {}: {}", filename, err);
        process::exit(1);
    });

    // 4. Initialize the Scanner with the file content
    // Note: 'content' lives for the rest of main(), 
    // satisfying the Scanner's lifetime requirement.
    let scanner = Scanner::new(&content);

    // 5. Iterate and print
    for result in scanner {
        match result {
            Ok(spanned_token) => {
                println!(
                    "[{:?}] {:<15} @ {}:{}", 
                    spanned_token.span.start.offset,
                    format!("{:?}", spanned_token.token),
                    spanned_token.span.start.line,
                    spanned_token.span.start.column
                );
            },
            Err(e) => {
                eprintln!("Lexical Error: {:?}", e);
            }
        }
    }
}