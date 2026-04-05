# b compiler

A compiler for the B programming language — Ken Thompson and Dennis Ritchie's predecessor to C — written in Rust. Implements a complete front-end pipeline (lexer → parser → AST) with a visitor-based pass architecture, string interning, and a partially complete semantic analysis stage. Benchmarks available at [brian-wang07.github.io/b-compiler](https://brian-wang07.github.io/b-compiler/).

---

## architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          SOURCE FILE (.b)                           │
└────────────────────────────┬────────────────────────────────────────┘
                             │  &str  (zero-copy lifetime 'a)
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                            LEXER                                    │
│                                                                     │
│   Scanner<'a>                                                       │
│     ├── char-by-char iteration with one-character peek             │
│     ├── Span tracking  (u32 byte offsets, binary-search newlines)  │
│     └── produces  SpannedToken<'a>  (Token + Span)                 │
│                                                                     │
│   Token variants                                                    │
│     Keyword | Operator | Delimiter | Identifier | Integer          │
│     StringLiteral | CharLiteral | EOF                              │
└────────────────────────────┬────────────────────────────────────────┘
                             │  &[SpannedToken<'a>]
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                           PARSER                                    │
│                                                                     │
│   Top-level  →  recursive descent                                  │
│   Expressions →  Pratt parser (top-down operator precedence)       │
│   Statements  →  recursive descent                                 │
│                                                                     │
│   Produces a borrowed AST — all name slices point into the         │
│   original source buffer; no heap allocation for identifiers       │
└────────────────────────────┬────────────────────────────────────────┘
                             │  Program<'a>  (Item tree)
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                             AST                                     │
│                                                                     │
│   Program                                                          │
│     └── Item*                                                       │
│           ├── Function  { name, params[], body: Stmt }             │
│           └── Global    { name, size?, initializer? }[]            │
│                                                                     │
│   Stmt: Block | Auto | Extrn | Expression | If | While | Switch    │
│          Case | Default | Label | Goto | Return | Null             │
│                                                                     │
│   Expr: Assign | Binary | Call | Grouping | Literal | Unary       │
│          Variable | Get | Ternary | Postfix                        │
└────────────────────────────┬────────────────────────────────────────┘
                             │  &Program<'a>
              ┌──────────────┴──────────────┐
              ▼                             ▼
┌─────────────────────────┐   ┌─────────────────────────────────────┐
│     PRETTY PRINTER      │   │         SEMANTIC ANALYSIS           │
│                         │   │                                     │
│  AstPrinter             │   │  SymbolTable                        │
│  ExprVisitor<String>    │   │    ├── Interner  (FxHashMap)        │
│  StmtVisitor<String>    │   │    └── GlobalEnv / FunctionEnv      │
│  ItemVisitor<String>    │   │                                     │
│                         │   │  SymbolKind:                        │
│  emits S-expressions    │   │    Auto | Extrn | Param             │
│  with indentation       │   │    Function | Label                 │
└─────────────────────────┘   └───────────────┬─────────────────────┘
                                              │  (in progress)
                                              ▼
                              ┌─────────────────────────────────────┐
                              │             IR  /  CODEGEN          │
                              │                                     │
                              │             (planned)               │
                              └─────────────────────────────────────┘
```

---

## lexer

The scanner is a single-pass, zero-copy tokenizer. It holds a `&str` reference to the source and produces `SpannedToken<'a>` values without allocating string copies for identifiers.

### token types

| variant | description |
|---|---|
| `Keyword(Keyword)` | reserved words: `auto extrn if else while switch case default return goto` |
| `Operator(Operator)` | arithmetic, bitwise, comparison, increment/decrement, assignment |
| `Delimiter(Delimiter)` | `( ) [ ] { } , ; : ?` |
| `Identifier(&'a str)` | zero-copy slice into source |
| `Integer(i64)` | decimal or octal (leading `0`) numeric literal |
| `StringLiteral(String)` | heap-allocated with escape processing |
| `CharLiteral(i64)` | 1–2 characters packed as `(c1 << 8) | c2` |
| `EOF` | sentinel |

### operators

| category | tokens |
|---|---|
| arithmetic | `+  -  *  /  %` |
| bitwise | `&  \|  ^  ~  <<  >>` |
| comparison | `==  !=  <  >  <=  >=` |
| logical | `!` |
| assignment | `=  =+  =-  =*  =/  =%  =&` |
| increment/decrement | `++  --` (prefix and postfix) |

Compound assignments use the reversed B syntax — `=+` rather than `+=`.

### escape sequences

B uses `*` as the escape character (not `\`):

| sequence | meaning |
|---|---|
| `*n` | newline |
| `*t` | tab |
| `*0` | null byte |
| `**` | literal `*` |
| `*"` | literal `"` |

### span tracking

```rust
pub struct Span {
    pub start: u32,   // byte offset into source
    pub end:   u32,
}
```

The scanner maintains a `cols: Vec<u32>` of newline positions. Line/column numbers for diagnostics are computed on demand via binary search — the hot path pays zero overhead for position tracking.

---

## parser

### expression parsing — pratt parser

Expressions are parsed with a Pratt (top-down operator precedence) parser. Each operator carries two binding powers: left (how tightly it binds to the left operand) and right (how tightly it binds to the right).

| precedence level | operators | associativity |
|---|---|---|
| Assignment | `=  =+  =-  =*  =/  =%  =&` | right |
| Ternary | `?:` | right |
| BitOr | `\|` | left |
| BitXor | `^` | left |
| BitAnd | `&` | left |
| Equality | `==  !=` | left |
| Comparison | `<  >  <=  >=` | left |
| Shift | `<<  >>` | left |
| Addition | `+  -` | left |
| Multiplication | `*  /  %` | left |
| Unary (prefix) | `-  !  ~  &  *  ++  --` | right |
| Postfix | `++  --  []  ()` | left |

Prefix positions (`nud`) handle primary expressions and right-associative unary operators. Infix/postfix positions (`led`) extend a completed left sub-expression.

### statement parsing — recursive descent

| statement | syntax |
|---|---|
| block | `{ stmt* }` |
| auto | `auto name[size]?, ... ;` — must appear first in block |
| extrn | `extrn name, ... ;` |
| if/else | `if ( expr ) stmt ( else stmt )?` |
| while | `while ( expr ) stmt` |
| switch | `switch ( expr ) stmt` |
| case | `case expr : stmt` |
| default | `default : stmt` |
| label | `name : stmt` |
| goto | `goto expr ;` |
| return | `return ( expr )? ;` |
| null | `;` |

`auto` declarations are scoped to the enclosing function (not the block). The parser enforces that `auto` appears before any other statement and raises `AutoRedecl` otherwise. `extrn` resets this gate — it may follow auto declarations and precede additional ones.

### declarations

```
// global
x;                          // scalar, no initializer
x 42;                       // scalar, initialized
v[10];                      // array, size 10
v[3] 1, 2, 3;              // array with initializer list
a, b, c;                    // multiple scalars

// local
auto x;
auto buf[64];
auto x, buf[64], y;

// external
extrn printf, exit, getchar;
```

---

## ast

All AST nodes borrow from the token slice produced by the lexer. No AST node allocates a new string — identifiers are `&'a str` slices pointing into the original source.

### expressions

```rust
pub enum Expr<'a> {
    Assign   { lvalue: Box<Expr<'a>>, operator: &'a SpannedToken<'a>, value: Box<Expr<'a>> },
    Binary   { left: Box<Expr<'a>>,   operator: &'a SpannedToken<'a>, right: Box<Expr<'a>> },
    Call     { callee: Box<Expr<'a>>, arguments: Vec<Expr<'a>> },
    Grouping { expression: Box<Expr<'a>> },
    Literal  { value: Lexeme<'a> },
    Unary    { operator: &'a SpannedToken<'a>, right: Box<Expr<'a>> },
    Variable { name: &'a SpannedToken<'a> },
    Get      { target: Box<Expr<'a>>, index: Box<Expr<'a>> },
    Ternary  { condition: Box<Expr<'a>>, then_branch: Box<Expr<'a>>, else_branch: Box<Expr<'a>> },
    Postfix  { left: Box<Expr<'a>>, operator: &'a SpannedToken<'a> },
}
```

### statements

```rust
pub enum Stmt<'a> {
    Block      { statements: Vec<Stmt<'a>> },
    Auto       { declarations: Vec<AutoDecl<'a>> },
    Extrn      { names: Vec<&'a SpannedToken<'a>> },
    Expression { expression: Expr<'a> },
    If         { condition: Expr<'a>, then_branch: Box<Stmt<'a>>, else_branch: Option<Box<Stmt<'a>>> },
    While      { condition: Expr<'a>, body: Box<Stmt<'a>> },
    Switch     { condition: Expr<'a>, cases: Box<Stmt<'a>> },
    Case       { value: Expr<'a>, body: Box<Stmt<'a>> },
    Default    { body: Box<Stmt<'a>> },
    Label      { name: &'a SpannedToken<'a>, body: Box<Stmt<'a>> },
    Goto       { expression: Expr<'a> },
    Return     { value: Option<Expr<'a>> },
    Null,
}
```

### top-level items

```rust
pub struct Function<'a> {
    pub name:   &'a SpannedToken<'a>,
    pub params: Vec<&'a SpannedToken<'a>>,
    pub body:   Box<Stmt<'a>>,
}

pub struct GlobalDecl<'a> {
    pub name:        &'a SpannedToken<'a>,
    pub size:        Option<&'a SpannedToken<'a>>,
    pub initializer: Option<Vec<&'a SpannedToken<'a>>>,
}
```

---

## visitor pattern

The AST exposes three visitor traits, each generic over a return type `T`. Concrete passes implement one or more of these and call the corresponding `walk_*` function to recurse.

```rust
pub trait ExprVisitor<T> {
    fn visit_assign(&mut self, lvalue: &Expr, op: &SpannedToken, value: &Expr) -> T;
    fn visit_binary(&mut self, left: &Expr, op: &SpannedToken, right: &Expr) -> T;
    fn visit_call(&mut self, callee: &Expr, args: &[Expr]) -> T;
    fn visit_grouping(&mut self, expr: &Expr) -> T;
    fn visit_literal(&mut self, value: &Lexeme) -> T;
    fn visit_unary(&mut self, op: &SpannedToken, right: &Expr) -> T;
    fn visit_variable(&mut self, name: &SpannedToken) -> T;
    fn visit_get(&mut self, target: &Expr, index: &Expr) -> T;
    fn visit_ternary(&mut self, cond: &Expr, then: &Expr, else_: &Expr) -> T;
    fn visit_postfix(&mut self, left: &Expr, op: &SpannedToken) -> T;
}

pub trait StmtVisitor<T> {
    fn visit_block(&mut self, stmts: &[Stmt]) -> T;
    fn visit_auto(&mut self, decls: &[AutoDecl]) -> T;
    fn visit_extrn(&mut self, names: &[&SpannedToken]) -> T;
    fn visit_expression(&mut self, expr: &Expr) -> T;
    fn visit_if(&mut self, cond: &Expr, then: &Stmt, else_: Option<&Stmt>) -> T;
    fn visit_while(&mut self, cond: &Expr, body: &Stmt) -> T;
    fn visit_switch(&mut self, cond: &Expr, cases: &Stmt) -> T;
    fn visit_case(&mut self, value: &Expr, body: &Stmt) -> T;
    fn visit_default(&mut self, body: &Stmt) -> T;
    fn visit_label(&mut self, name: &SpannedToken, body: &Stmt) -> T;
    fn visit_goto(&mut self, expr: &Expr) -> T;
    fn visit_return(&mut self, value: Option<&Expr>) -> T;
    fn visit_null(&mut self) -> T;
}

pub trait ItemVisitor<T> {
    fn visit_function(&mut self, func: &Function) -> T;
    fn visit_global(&mut self, decls: &[GlobalDecl]) -> T;
}
```

Walk functions dispatch to the correct method and recurse:

```rust
pub fn walk_expr<T, V: ExprVisitor<T>>(visitor: &mut V, expr: &Expr) -> T;
pub fn walk_stmt<T, V: StmtVisitor<T>>(visitor: &mut V, stmt: &Stmt) -> T;
pub fn walk_item<T, V: ItemVisitor<T>>(visitor: &mut V, item: &Item) -> T;
```

The generic `T` allows passes to accumulate results (e.g. `String` for printing, `bool` for liveness, `()` for mutation) without virtual dispatch.

---

## pretty printer

`AstPrinter` implements all three visitor traits and emits a parenthesized S-expression representation of the AST — useful for debugging and test assertions.

```
a + b * c              →   (+ a (* b c))
x[i]                   →   (get x i)
f(a, b)                →   (call f a b)
a =+ b                 →   (=+ a b)
a ? b : c              →   (ternary a b c)

main(argc, argv) {     →   (fn main (argc argv)
  auto x;                    (block
  x = 1;                       (auto x)
  return x;                    (= x 1)
}                              (return x)))
```

---

## semantic analysis

The symbol table is structured around two-level scoping (global + per-function). B does not support nested block scopes — all `auto` locals within a function share a single flat frame.

### string interning

All identifiers are deduplicated through an interner before they enter the symbol table:

```rust
pub struct Interner {
    strings: Vec<String>,
    lookup:  FxHashMap<String, SymbolId>,
}

pub struct SymbolId(u32);   // 4-byte handle replaces &str everywhere downstream
```

`FxHashMap` (rustc-hash) is used throughout the symbol table for its fast non-cryptographic hashing of short strings.

### symbol kinds and locations

```rust
pub enum SymbolKind { Auto, Extrn, Param, Function, Label }

pub enum Location {
    Local    { slot: u32 },           // stack frame index
    Global   { index: u32 },          // global data table
    Function { func_index: u32 },     // function table
    Import   { import_index: u32 },   // external symbol table
}

pub struct Symbol {
    name: SymbolId,
    kind: SymbolKind,
    size: u32,       // 1 for scalars, N for arrays
    slot: Location,
}
```

### scope lookup

```rust
pub struct SymbolTable {
    interner: Interner,
    global:   GlobalEnv,    // FxHashMap<SymbolId, Symbol>
}

pub struct FunctionEnv {
    locals: FxHashMap<SymbolId, Symbol>,
    params: Vec<SymbolId>,
}
```

Resolution order: function locals/params → global. Each function is analyzed with a fresh `FunctionEnv`; the `SymbolTable` provides the global backing store.

---

## cli tools

| binary | usage | description |
|---|---|---|
| `lex` | `cargo run --bin lex <file.b>` | tokenize and dump `[offset] Token` for every token |
| `printer` | `cargo run --bin printer <file.b>` | lex + parse, print debug AST and S-expression tree |

---

## error types

### lexer errors

| error | cause |
|---|---|
| `UnexpectedChar(char, Span)` | character not valid in any token |
| `UnterminatedString(Span)` | `"` with no closing quote |
| `UnterminatedChar(Span)` | `'` with no closing `'` |
| `UnterminatedComment(Span)` | `/*` with no `*/` |
| `InvalidNumber(String, Span)` | malformed numeric literal |

### parser errors

| error | cause |
|---|---|
| `UnexpectedToken(Expected)` | token mismatch against grammar |
| `UnknownToken(&SpannedToken)` | token illegal in this position |
| `UnexpectedEOF` | input ended mid-production |
| `RValueAssign` | assignment target is not an lvalue |
| `AutoRedecl` | `auto` declaration after non-declaration statement |
| `UnspecifiedArraySizeInitialization` | initializer list with no declared array size |

---

## roadmap

### complete
- Zero-copy lexer with `*`-escape strings, packed char literals, and octal integers
- Pratt expression parser with full B operator set and compound assignments
- Recursive-descent statement parser covering all B constructs including `goto`/labels
- Borrowed AST — no identifier copies, all slices point into the source buffer
- Visitor traits (`ExprVisitor`, `StmtVisitor`, `ItemVisitor`) with generic return type
- S-expression pretty printer via visitor pattern
- Symbol table structure with string interning (`FxHashMap`-backed `Interner`)
- CLI tools: tokenizer dump (`lex`), AST printer (`printer`)
- Criterion benchmarks for lexer, parser, and memory usage
- 170+ edge case tests covering lexer, parser, and end-to-end pipeline

### in progress
- Semantic analysis pass: symbol resolution, duplicate declaration detection
- Connecting `FunctionEnv` walk to the visitor pipeline
- Diagnostic error reporting with line/column from `Span`

### planned: IR and codegen
- Lowering AST to a typed three-address IR (SSA or linear)
- `goto`/label resolution — build a CFG from the IR
- Register allocation
- Codegen target (x86-64 or WASM)
- Linker integration for `extrn` declarations

---

## build

```sh
cargo build --release
```

Requires Rust 2024 edition (stable ≥ 1.85).

---

## running

```sh
# dump tokens
cargo run --bin lex examples/hello.b

# dump AST and S-expression tree
cargo run --bin printer examples/hello.b

# run tests
cargo test

# run benchmarks
cargo bench
```
