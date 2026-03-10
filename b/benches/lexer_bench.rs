use criterion::{criterion_group, criterion_main, Criterion, black_box};
use b::lexer::scanner::Scanner;

/// Helper: lex a source string, collecting all tokens.
fn lex_all(source: &str) -> Vec<b::lexer::token::SpannedToken<'_>> {
    Scanner::new(source)
        .collect::<Result<Vec<_>, _>>()
        .expect("lex error in benchmark input")
}

// ---------------------------------------------------------------------------
//  Inputs
// ---------------------------------------------------------------------------

/// A minimal program.
fn tiny_source() -> String {
    "main() { auto a; a = 1; }".into()
}

/// A medium program with diverse tokens.
fn medium_source() -> String {
    let mut s = String::new();
    for i in 0..200 {
        s.push_str(&format!(
            "f{}() {{ auto a, b, c[10]; a = {}; b =+ a; c[0] = a + b * 2; if (a == b) a++; }}\n",
            i, i
        ));
    }
    s
}

/// A large program that stresses the lexer with many tokens.
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

/// Stress-test: very long identifiers.
fn long_identifiers() -> String {
    let mut s = String::new();
    for i in 0..500 {
        let name: String = format!("var_{}", "a".repeat(200));
        s.push_str(&format!("{}_{};", name, i));
    }
    // Wrap in a function so it's valid top-level B.
    format!("main() {{ auto x; {} }}", s)
}

/// Stress-test: many string literals with escape sequences.
fn many_strings() -> String {
    let mut s = String::new();
    for _ in 0..500 {
        s.push_str(r#"x = "hello*nworld*ttab*0null**star";"#);
        s.push('\n');
    }
    format!("main() {{ auto x; {} }}", s)
}

/// Stress-test: deeply nested comments.
fn many_comments() -> String {
    let mut s = String::new();
    for _ in 0..500 {
        s.push_str("/* comment */ ");
    }
    s.push_str("main() { auto x; x = 1; }");
    s
}

/// Stress-test: many numeric literals (decimal + octal).
fn many_numbers() -> String {
    let mut s = String::new();
    s.push_str("main() { auto x;\n");
    for i in 0..1000 {
        if i % 2 == 0 {
            s.push_str(&format!("x = {};\n", i));
        } else {
            s.push_str(&format!("x = 0{};\n", i)); // octal
        }
    }
    s.push_str("}");
    s
}

// ---------------------------------------------------------------------------
//  Benchmarks
// ---------------------------------------------------------------------------

fn bench_lex_tiny(c: &mut Criterion) {
    let src = tiny_source();
    c.bench_function("lex/tiny", |b| {
        b.iter(|| lex_all(black_box(&src)))
    });
}

fn bench_lex_medium(c: &mut Criterion) {
    let src = medium_source();
    c.bench_function("lex/medium (200 fns)", |b| {
        b.iter(|| lex_all(black_box(&src)))
    });
}

fn bench_lex_large(c: &mut Criterion) {
    let src = large_source();
    c.bench_function("lex/large (1000 fns)", |b| {
        b.iter(|| lex_all(black_box(&src)))
    });
}

fn bench_lex_long_identifiers(c: &mut Criterion) {
    let src = long_identifiers();
    c.bench_function("lex/long_identifiers", |b| {
        b.iter(|| lex_all(black_box(&src)))
    });
}

fn bench_lex_many_strings(c: &mut Criterion) {
    let src = many_strings();
    c.bench_function("lex/many_strings", |b| {
        b.iter(|| lex_all(black_box(&src)))
    });
}

fn bench_lex_many_comments(c: &mut Criterion) {
    let src = many_comments();
    c.bench_function("lex/many_comments", |b| {
        b.iter(|| lex_all(black_box(&src)))
    });
}

fn bench_lex_many_numbers(c: &mut Criterion) {
    let src = many_numbers();
    c.bench_function("lex/many_numbers", |b| {
        b.iter(|| lex_all(black_box(&src)))
    });
}

criterion_group!(
    benches,
    bench_lex_tiny,
    bench_lex_medium,
    bench_lex_large,
    bench_lex_long_identifiers,
    bench_lex_many_strings,
    bench_lex_many_comments,
    bench_lex_many_numbers,
);
criterion_main!(benches);
