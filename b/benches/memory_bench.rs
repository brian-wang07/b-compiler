//! Memory-profiling benchmarks using dhat.
//!
//! Run with:
//!   cargo bench --bench memory_bench
//!
//! dhat will print a summary of heap allocations to stderr.
//! For a full viewer, set `DHAT_OUT_FILE=dhat-heap.json` and open in
//! https://nnethercote.github.io/dh_view/dh_view.html

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use b::lexer::scanner::Scanner;
use b::lexer::token::{SpannedToken, Token};
use b::parser::Parser;

// ---------------------------------------------------------------------------
//  Helpers
// ---------------------------------------------------------------------------

fn lex(source: &str) -> Vec<SpannedToken<'_>> {
    let mut tokens: Vec<_> = Scanner::new(source)
        .collect::<Result<Vec<_>, _>>()
        .expect("lex error");
    tokens.push(SpannedToken {
        token: Token::EOF,
        span: b::common::span::Span::default(),
    });
    tokens
}

// ---------------------------------------------------------------------------
//  Source generators
// ---------------------------------------------------------------------------

fn large_source() -> String {
    let mut s = String::new();
    for i in 0..1000 {
        s.push_str(&format!(
            "f{}() {{ auto x, y; x = {}; y = x + {} * (x - {}); if (x >= y) x =+ y; while (x) x--; }}\n",
            i, i, i + 1, i + 2
        ));
    }
    s
}

fn deep_expression_source() -> String {
    let depth = 300;
    let mut expr = String::from("x");
    for i in 0..depth {
        expr = format!("({} + {})", expr, i);
    }
    format!("main() {{ auto x; x = {}; }}", expr)
}

fn many_strings_source() -> String {
    let mut s = String::new();
    for _ in 0..500 {
        s.push_str(r#"x = "hello*nworld*ttab*0null**star";"#);
        s.push('\n');
    }
    format!("main() {{ auto x; {} }}", s)
}

fn many_globals_source() -> String {
    let mut s = String::new();
    for i in 0..300 {
        s.push_str(&format!("g{}[4] {},{},{},{};\n", i, i, i + 1, i + 2, i + 3));
    }
    s.push_str("main() { auto x; x = 0; }\n");
    s
}

// ---------------------------------------------------------------------------
//  Profiling scenarios
// ---------------------------------------------------------------------------

fn profile_lex_large(source: &str) {
    let _tokens: Vec<_> = Scanner::new(source)
        .collect::<Result<Vec<_>, _>>()
        .expect("lex error");
}

fn profile_parse_large(source: &str) {
    let tokens = lex(source);
    let mut parser = Parser::new(&tokens);
    let _ = parser.parse_program();
}

fn profile_e2e(source: &str) {
    let tokens = lex(source);
    let mut parser = Parser::new(&tokens);
    let _ = parser.parse_program();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scenario = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    let _profiler = dhat::Profiler::new_heap();

    match scenario {
        "lex" => {
            eprintln!("=== dhat: lex large (1000 fns) ===");
            let src = large_source();
            profile_lex_large(&src);
        }
        "parse" => {
            eprintln!("=== dhat: parse large (1000 fns) ===");
            let src = large_source();
            profile_parse_large(&src);
        }
        "deep_expr" => {
            eprintln!("=== dhat: parse deep expression (300 deep) ===");
            let src = deep_expression_source();
            profile_parse_large(&src);
        }
        "strings" => {
            eprintln!("=== dhat: lex many strings (500) ===");
            let src = many_strings_source();
            profile_lex_large(&src);
        }
        "globals" => {
            eprintln!("=== dhat: parse many globals (300) ===");
            let src = many_globals_source();
            profile_parse_large(&src);
        }
        "all" | _ => {
            eprintln!("=== dhat: end-to-end large (1000 fns) ===");
            let src = large_source();
            profile_e2e(&src);
        }
    }

    // dhat profiler drops here and prints the summary.
}
