use criterion::{criterion_group, criterion_main, Criterion, black_box};
use b::lexer::scanner::Scanner;
use b::lexer::token::SpannedToken;
use b::parser::Parser;

// ---------------------------------------------------------------------------
//  Helpers
// ---------------------------------------------------------------------------

fn lex(source: &str) -> Vec<SpannedToken<'_>> {
    let mut tokens: Vec<_> = Scanner::new(source)
        .collect::<Result<Vec<_>, _>>()
        .expect("lex error in benchmark input");
    // Parser expects an EOF sentinel at the end.
    tokens.push(SpannedToken {
        token: b::lexer::token::Token::EOF,
        span: b::common::span::Span::default(),
    });
    tokens
}

fn parse_program(source: &str) {
    let tokens = lex(source);
    let mut parser = Parser::new(&tokens);
    let _ = parser.parse_program(); // ignore errors for stress benchmarks
}

// ---------------------------------------------------------------------------
//  Source generators
// ---------------------------------------------------------------------------

fn tiny_source() -> String {
    "main() { auto a; a = 1; }".into()
}

fn medium_source() -> String {
    let mut s = String::new();
    for i in 0..200 {
        s.push_str(&format!(
            "f{}() {{ auto a, b; a = {}; b = a + 1; if (a == b) a = b; }}\n",
            i, i
        ));
    }
    s
}

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

/// Deeply nested if-else chains stress the recursive descent parser.
fn deep_nesting_source() -> String {
    let depth = 200;
    let mut s = String::from("main() {\n  auto x;\n  x = 0;\n");
    for _ in 0..depth {
        s.push_str("  if (x) {\n");
    }
    s.push_str("    x = 1;\n");
    for _ in 0..depth {
        s.push_str("  }\n");
    }
    s.push_str("}\n");
    s
}

/// Deeply nested expressions: ((((((x + 1) + 2) + 3) ...)))
fn deep_expression_source() -> String {
    let depth = 300;
    let mut expr = String::from("x");
    for i in 0..depth {
        expr = format!("({} + {})", expr, i);
    }
    format!("main() {{ auto x; x = {}; }}", expr)
}

/// Many function calls with arguments — stresses call-expression parsing.
fn many_calls_source() -> String {
    let mut s = String::new();
    s.push_str("main() {\n  auto x;\n");
    for i in 0..500 {
        s.push_str(&format!("  x = f(x, {}, x + {});\n", i, i + 1));
    }
    s.push_str("}\n");
    // Define a stub so top-level parsing works.
    s.push_str("f(a, b, c) { return (a); }\n");
    s
}

/// Many global declarations with array initializers.
fn many_globals_source() -> String {
    let mut s = String::new();
    for i in 0..300 {
        s.push_str(&format!("g{}[4] {},{},{},{};\n", i, i, i + 1, i + 2, i + 3));
    }
    s.push_str("main() { auto x; x = 0; }\n");
    s
}

/// Ternary chains: a ? b ? c ? ... : ... : ... : ...
fn ternary_chain_source() -> String {
    let depth = 100;
    let mut expr = String::from("0");
    for i in (0..depth).rev() {
        expr = format!("(x ? {} : {})", i, expr);
    }
    format!("main() {{ auto x; x = {}; }}", expr)
}

// ---------------------------------------------------------------------------
//  Benchmarks: parser only (pre-lexed input)
// ---------------------------------------------------------------------------

fn bench_parse_tiny(c: &mut Criterion) {
    let src = tiny_source();
    let tokens = lex(&src);
    c.bench_function("parse/tiny", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&tokens));
            let _ = parser.parse_program();
        })
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    let src = medium_source();
    let tokens = lex(&src);
    c.bench_function("parse/medium (200 fns)", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&tokens));
            let _ = parser.parse_program();
        })
    });
}

fn bench_parse_large(c: &mut Criterion) {
    let src = large_source();
    let tokens = lex(&src);
    c.bench_function("parse/large (1000 fns)", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&tokens));
            let _ = parser.parse_program();
        })
    });
}

fn bench_parse_deep_nesting(c: &mut Criterion) {
    let src = deep_nesting_source();
    let tokens = lex(&src);
    c.bench_function("parse/deep_nesting (200)", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&tokens));
            let _ = parser.parse_program();
        })
    });
}

fn bench_parse_deep_expression(c: &mut Criterion) {
    let src = deep_expression_source();
    let tokens = lex(&src);
    c.bench_function("parse/deep_expression (300)", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&tokens));
            let _ = parser.parse_program();
        })
    });
}

fn bench_parse_many_calls(c: &mut Criterion) {
    let src = many_calls_source();
    let tokens = lex(&src);
    c.bench_function("parse/many_calls (500)", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&tokens));
            let _ = parser.parse_program();
        })
    });
}

fn bench_parse_many_globals(c: &mut Criterion) {
    let src = many_globals_source();
    let tokens = lex(&src);
    c.bench_function("parse/many_globals (300)", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&tokens));
            let _ = parser.parse_program();
        })
    });
}

fn bench_parse_ternary_chain(c: &mut Criterion) {
    let src = ternary_chain_source();
    let tokens = lex(&src);
    c.bench_function("parse/ternary_chain (100)", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&tokens));
            let _ = parser.parse_program();
        })
    });
}

// ---------------------------------------------------------------------------
//  Benchmarks: lex + parse (end-to-end)
// ---------------------------------------------------------------------------

fn bench_e2e_medium(c: &mut Criterion) {
    let src = medium_source();
    c.bench_function("e2e/medium (200 fns)", |b| {
        b.iter(|| parse_program(black_box(&src)))
    });
}

fn bench_e2e_large(c: &mut Criterion) {
    let src = large_source();
    c.bench_function("e2e/large (1000 fns)", |b| {
        b.iter(|| parse_program(black_box(&src)))
    });
}

criterion_group!(
    benches,
    bench_parse_tiny,
    bench_parse_medium,
    bench_parse_large,
    bench_parse_deep_nesting,
    bench_parse_deep_expression,
    bench_parse_many_calls,
    bench_parse_many_globals,
    bench_parse_ternary_chain,
    bench_e2e_medium,
    bench_e2e_large,
);
criterion_main!(benches);
