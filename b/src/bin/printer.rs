use std::env;
use std::fs;

use b::lexer::scanner::Scanner;
use b::parser::Parser;
use b::ast::pretty_printer::AstPrinter;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // If no file provided, use a test expression
        let test_expr = "a + b * c";
        println!("No file provided. Testing with: {}", test_expr);
        run(test_expr);
        return;
    }

    let filename = &args[1];
    let source = fs::read_to_string(filename).expect("Failed to read file");
    run(&source);
}

fn run(source: &str) {
    let scanner = Scanner::new(source);
    let mut toks = Vec::new();
    for token in scanner {
      match token {
        Ok(token) => toks.push(token),
        Err(e) => {
          eprintln!("Lexer error: {:?}", e);
          return;
        }
      }
    }

    let mut parser = Parser::new(&toks);

    match parser.parse_program() {
        Ok(program) => {
            println!("AST:\n{:#?}\n", program);
            let mut printer = AstPrinter::new();
            let output = printer.print_program(&program);
            println!("Formatted:\n{}", output);
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
        }
    }
}