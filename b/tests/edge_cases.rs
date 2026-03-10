//! Edge-case unit tests for the lexer, parser, and lexer+parser pipeline.
//!
//! Focus areas: inputs that could cause blow-ups in time, memory, or
//! hardware pressure (deep recursion, huge token counts, pathological
//! strings, numeric edge cases, etc.).

#[cfg(test)]
mod lexer_tests {
    use b::lexer::scanner::Scanner;
    use b::lexer::token::*;
    use b::lexer::LexError;

    /// Convenience: lex and collect tokens (unwrapping errors).
    fn lex_ok(src: &str) -> Vec<SpannedToken<'_>> {
        Scanner::new(src)
            .collect::<Result<Vec<_>, _>>()
            .expect("unexpected lex error")
    }

    /// Lex and collect errors.
    fn lex_errors(src: &str) -> Vec<LexError> {
        Scanner::new(src)
            .filter_map(|r| r.err())
            .collect()
    }

    // ---- empty / whitespace-only inputs ----

    #[test]
    fn empty_input_produces_no_tokens() {
        assert!(lex_ok("").is_empty());
    }

    #[test]
    fn whitespace_only_produces_no_tokens() {
        assert!(lex_ok("   \n\t\r\n   ").is_empty());
    }

    // ---- comments ----

    #[test]
    fn unterminated_comment_is_error() {
        let errs = lex_errors("/* this never ends");
        assert!(!errs.is_empty());
        assert!(matches!(errs[0], LexError::UnterminatedComment(_)));
    }

    #[test]
    fn comment_with_stars_inside() {
        // Stars inside a comment that don't form */ should be fine.
        let tokens = lex_ok("/* * ** *** */ x");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token, Token::Identifier("x")));
    }

    #[test]
    fn adjacent_comments() {
        let tokens = lex_ok("/* a *//* b *//* c */ x");
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn comment_containing_slashes() {
        let tokens = lex_ok("/* // not a line comment */ x");
        assert_eq!(tokens.len(), 1);
    }

    // ---- numbers ----

    #[test]
    fn zero_is_valid() {
        let tokens = lex_ok("0");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token, Token::Integer(0)));
    }

    #[test]
    fn octal_number() {
        // 077 in octal = 63 decimal
        let tokens = lex_ok("077");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token, Token::Integer(63)));
    }

    #[test]
    fn large_decimal_number() {
        let tokens = lex_ok("999999999");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token, Token::Integer(999999999)));
    }

    #[test]
    fn negative_number_token() {
        let tokens = lex_ok("-42");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token, Token::Integer(-42)));
    }

    #[test]
    fn negative_zero() {
        let tokens = lex_ok("-0");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token, Token::Integer(0)));
    }

    #[test]
    fn many_leading_zeros_octal() {
        let tokens = lex_ok("00000");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token, Token::Integer(0)));
    }

    // ---- strings ----

    #[test]
    fn empty_string_literal() {
        let tokens = lex_ok(r#""""#);
        assert_eq!(tokens.len(), 1);
        match &tokens[0].token {
            Token::StringLiteral(s) => assert!(s.is_empty()),
            other => panic!("expected StringLiteral, got {:?}", other),
        }
    }

    #[test]
    fn string_with_all_escape_sequences() {
        let tokens = lex_ok(r#""*n*t*0****""#);
        assert_eq!(tokens.len(), 1);
        match &tokens[0].token {
            Token::StringLiteral(s) => {
                assert_eq!(s, "\n\t\0**");
            }
            other => panic!("expected StringLiteral, got {:?}", other),
        }
    }

    #[test]
    fn unterminated_string_is_error() {
        let errs = lex_errors(r#""hello world"#);
        assert!(!errs.is_empty());
        assert!(matches!(errs[0], LexError::UnterminatedString(_)));
    }

    #[test]
    fn string_with_escaped_star() {
        // ** is the escape for a literal *
        let tokens = lex_ok(r#""hello**world""#);
        assert_eq!(tokens.len(), 1);
        match &tokens[0].token {
            Token::StringLiteral(s) => assert_eq!(s, "hello*world"),
            other => panic!("expected StringLiteral, got {:?}", other),
        }
    }

    // ---- char literals ----

    #[test]
    fn single_char_literal() {
        let tokens = lex_ok("'a'");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token, Token::CharLiteral(97))); // 'a' = 97
    }

    #[test]
    fn two_char_literal_packing() {
        // B packs up to 2 chars: (first << 8) | second
        let tokens = lex_ok("'ab'");
        assert_eq!(tokens.len(), 1);
        match tokens[0].token {
            Token::CharLiteral(v) => {
                assert_eq!(v, (b'a' as i64) << 8 | (b'b' as i64));
            }
            _ => panic!("expected CharLiteral"),
        }
    }

    #[test]
    fn empty_char_literal() {
        let tokens = lex_ok("''");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token, Token::CharLiteral(0)));
    }

    #[test]
    fn unterminated_char_literal() {
        let errs = lex_errors("'abc");
        assert!(!errs.is_empty());
        assert!(matches!(errs[0], LexError::UnterminatedChar(_)));
    }

    #[test]
    fn char_literal_with_escape() {
        let tokens = lex_ok("'*n'");
        assert_eq!(tokens.len(), 1);
        match tokens[0].token {
            Token::CharLiteral(v) => assert_eq!(v, b'\n' as i64),
            _ => panic!("expected CharLiteral"),
        }
    }

    // ---- operators / delimiters ----

    #[test]
    fn all_compound_assignments() {
        let tokens = lex_ok("=+ =- =* =/ =% =&");
        let ops: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        assert!(matches!(ops[0], Token::Operator(Operator::AssignPlus)));
        assert!(matches!(ops[1], Token::Operator(Operator::AssignMinus)));
        assert!(matches!(ops[2], Token::Operator(Operator::AssignStar)));
        assert!(matches!(ops[3], Token::Operator(Operator::AssignSlash)));
        assert!(matches!(ops[4], Token::Operator(Operator::AssignPercent)));
        assert!(matches!(ops[5], Token::Operator(Operator::AssignAmp)));
    }

    #[test]
    fn shift_vs_comparison() {
        let tokens = lex_ok("<< >> <= >=");
        let ops: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        assert!(matches!(ops[0], Token::Operator(Operator::LShift)));
        assert!(matches!(ops[1], Token::Operator(Operator::RShift)));
        assert!(matches!(ops[2], Token::Operator(Operator::LessEq)));
        assert!(matches!(ops[3], Token::Operator(Operator::GreaterEq)));
    }

    #[test]
    fn increment_decrement_vs_arithmetic() {
        let tokens = lex_ok("++ + -- -1");
        let ops: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        assert!(matches!(ops[0], Token::Operator(Operator::Inc)));
        assert!(matches!(ops[1], Token::Operator(Operator::Plus)));
        assert!(matches!(ops[2], Token::Operator(Operator::Dec)));
        assert!(matches!(ops[3], Token::Integer(-1)));
    }

    #[test]
    fn assign_vs_equal() {
        let tokens = lex_ok("= ==");
        assert!(matches!(tokens[0].token, Token::Operator(Operator::Assign)));
        assert!(matches!(tokens[1].token, Token::Operator(Operator::Equal)));
    }

    // ---- keywords vs identifiers ----

    #[test]
    fn all_keywords() {
        let src = "auto extrn if else while switch case default return goto";
        let tokens = lex_ok(src);
        assert_eq!(tokens.len(), 10);
        assert!(matches!(tokens[0].token, Token::Keyword(Keyword::Auto)));
        assert!(matches!(tokens[9].token, Token::Keyword(Keyword::Goto)));
    }

    #[test]
    fn keyword_prefix_is_identifier() {
        // "auto" is a keyword but "automate" is an identifier.
        let tokens = lex_ok("automate iffy returning");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].token, Token::Identifier("automate")));
        assert!(matches!(tokens[1].token, Token::Identifier("iffy")));
        assert!(matches!(tokens[2].token, Token::Identifier("returning")));
    }

    // ---- unexpected characters ----

    #[test]
    fn unexpected_char_produces_error() {
        let errs = lex_errors("@");
        assert!(!errs.is_empty());
        assert!(matches!(errs[0], LexError::UnexpectedChar('@', _)));
    }

    #[test]
    fn dollar_sign_is_unexpected() {
        let errs = lex_errors("$");
        assert!(!errs.is_empty());
    }

    // ---- stress / edge case sizes ----

    #[test]
    fn very_long_identifier() {
        let name = "a".repeat(10_000);
        let tokens = lex_ok(&name);
        assert_eq!(tokens.len(), 1);
        match &tokens[0].token {
            Token::Identifier(s) => assert_eq!(s.len(), 10_000),
            _ => panic!("expected Identifier"),
        }
    }

    #[test]
    fn very_long_string_literal() {
        let body = "x".repeat(50_000);
        let src = format!(r#""{}""#, body);
        let tokens = lex_ok(&src);
        assert_eq!(tokens.len(), 1);
        match &tokens[0].token {
            Token::StringLiteral(s) => assert_eq!(s.len(), 50_000),
            _ => panic!("expected StringLiteral"),
        }
    }

    #[test]
    fn thousands_of_tokens_on_one_line() {
        // Many semicolons — no newlines.
        let src = ";".repeat(10_000);
        let tokens = lex_ok(&src);
        assert_eq!(tokens.len(), 10_000);
    }

    #[test]
    fn many_newlines_without_tokens() {
        let src = "\n".repeat(100_000);
        let tokens = lex_ok(&src);
        assert!(tokens.is_empty());
    }

    #[test]
    fn interleaved_comments_and_tokens() {
        let mut src = String::new();
        for i in 0..1000 {
            src.push_str(&format!("/* comment {} */ x ", i));
        }
        let tokens = lex_ok(&src);
        assert_eq!(tokens.len(), 1000);
    }

    #[test]
    fn minus_followed_by_identifier_is_two_tokens() {
        // -x should be Minus then Identifier, not a negative number.
        let tokens = lex_ok("-x");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].token, Token::Operator(Operator::Minus)));
        assert!(matches!(tokens[1].token, Token::Identifier("x")));
    }

    #[test]
    fn minus_followed_by_paren_is_two_tokens() {
        let tokens = lex_ok("-(1)");
        assert!(matches!(tokens[0].token, Token::Operator(Operator::Minus)));
        assert!(matches!(tokens[1].token, Token::Delimiter(Delimiter::LParen)));
    }

    #[test]
    fn span_tracking_across_newlines() {
        let tokens = lex_ok("a\nb\nc");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].span.start.line, 1);
        assert_eq!(tokens[1].span.start.line, 2);
        assert_eq!(tokens[2].span.start.line, 3);
    }
}

// ===========================================================================

#[cfg(test)]
mod parser_tests {
    use b::lexer::scanner::Scanner;
    use b::lexer::token::*;
    use b::parser::Parser;
    use b::ast::*;

    fn lex(src: &str) -> Vec<SpannedToken<'_>> {
        let mut tokens: Vec<_> = Scanner::new(src)
            .collect::<Result<Vec<_>, _>>()
            .expect("lex error in test input");
        tokens.push(SpannedToken {
            token: Token::EOF,
            span: b::common::span::Span::default(),
        });
        tokens
    }

    fn parse_succeeds(src: &str) -> bool {
        let tokens = lex(src);
        let mut parser = Parser::new(&tokens);
        parser.parse_program().is_ok()
    }

    fn parse_fails(src: &str) -> bool {
        !parse_succeeds(src)
    }

    /// Parse and return the number of top-level items.
    fn parse_item_count(src: &str) -> usize {
        let tokens = lex(src);
        let mut parser = Parser::new(&tokens);
        let prog = parser.parse_program().expect("parse failed");
        prog.items.len()
    }

    // ---- minimal programs ----

    #[test]
    fn empty_function() {
        assert_eq!(parse_item_count("main() ;"), 1);
        // Also check it's a function.
        let tokens = lex("main() ;");
        let mut parser = Parser::new(&tokens);
        let prog = parser.parse_program().unwrap();
        assert!(matches!(&prog.items[0], Item::Function(_)));
    }

    #[test]
    fn function_with_empty_block() {
        assert_eq!(parse_item_count("main() {}"), 1);
    }

    #[test]
    fn function_with_params() {
        let src = "add(a, b) { return (a + b); }";
        let tokens = lex(src);
        let mut parser = Parser::new(&tokens);
        let prog = parser.parse_program().unwrap();
        match &prog.items[0] {
            Item::Function(f) => assert_eq!(f.params.len(), 2),
            _ => panic!("expected function"),
        }
    }

    // ---- global declarations ----

    #[test]
    fn simple_global_variable() {
        let tokens = lex("x;");
        let mut parser = Parser::new(&tokens);
        let prog = parser.parse_program().unwrap();
        assert!(matches!(&prog.items[0], Item::Global(_)));
    }

    #[test]
    fn global_with_initializer() {
        let tokens = lex("x 42;");
        let mut parser = Parser::new(&tokens);
        let prog = parser.parse_program().unwrap();
        match &prog.items[0] {
            Item::Global(decls) => {
                assert_eq!(decls.len(), 1);
                assert!(decls[0].initializer.is_some());
            }
            _ => panic!("expected global"),
        }
    }

    #[test]
    fn global_array_with_initializers() {
        let tokens = lex("arr[3] 1, 2, 3;");
        let mut parser = Parser::new(&tokens);
        let prog = parser.parse_program().unwrap();
        match &prog.items[0] {
            Item::Global(decls) => {
                assert!(decls[0].size.is_some());
                assert_eq!(decls[0].initializer.as_ref().unwrap().len(), 3);
            }
            _ => panic!("expected global"),
        }
    }

    #[test]
    fn global_array_no_initializer() {
        let tokens = lex("arr[10];");
        let mut parser = Parser::new(&tokens);
        let prog = parser.parse_program().unwrap();
        match &prog.items[0] {
            Item::Global(decls) => {
                assert!(decls[0].size.is_some());
                assert!(decls[0].initializer.is_none());
            }
            _ => panic!("expected global"),
        }
    }

    #[test]
    fn multiple_globals_comma_separated() {
        let tokens = lex("a, b, c;");
        let mut parser = Parser::new(&tokens);
        let prog = parser.parse_program().unwrap();
        match &prog.items[0] {
            Item::Global(decls) => assert_eq!(decls.len(), 3),
            _ => panic!("expected global"),
        }
    }

    // ---- expressions ----

    #[test]
    fn binary_operator_precedence() {
        assert!(parse_succeeds("main() { a + b * c; }"));
    }

    #[test]
    fn left_associativity() {
        assert!(parse_succeeds("main() { a - b - c; }"));
    }

    #[test]
    fn right_associativity_assignment() {
        assert!(parse_succeeds("main() { a = b = c; }"));
    }

    #[test]
    fn ternary_expression() {
        assert!(parse_succeeds("main() { a ? b : c; }"));
    }

    #[test]
    fn nested_ternary() {
        assert!(parse_succeeds("main() { a ? b ? c : d : e; }"));
    }

    #[test]
    fn function_call_no_args() {
        assert!(parse_succeeds("main() { f(); }"));
    }

    #[test]
    fn function_call_many_args() {
        assert!(parse_succeeds("main() { f(a, b, c, d, e); }"));
    }

    #[test]
    fn array_index_expression() {
        assert!(parse_succeeds("main() { a[0]; }"));
    }

    #[test]
    fn nested_array_index() {
        assert!(parse_succeeds("main() { a[b[c[0]]]; }"));
    }

    #[test]
    fn prefix_unary_operators() {
        assert!(parse_succeeds("main() { -a; !b; ~c; ++d; --e; &f; *g; }"));
    }

    #[test]
    fn postfix_operators() {
        assert!(parse_succeeds("main() { a++; b--; }"));
    }

    #[test]
    fn grouping_parens() {
        assert!(parse_succeeds("main() { (a + b) * c; }"));
    }

    #[test]
    fn deeply_nested_parens() {
        let depth = 50;
        let open: String = "(".repeat(depth);
        let close: String = ")".repeat(depth);
        let src = format!("main() {{ {}a{}; }}", open, close);
        assert!(parse_succeeds(&src));
    }

    // ---- statements ----

    #[test]
    fn auto_declaration_multiple() {
        assert!(parse_succeeds("main() { auto a, b, c[10]; a = 1; }"));
    }

    #[test]
    fn extrn_declaration() {
        assert!(parse_succeeds("main() { extrn putchar, getchar; putchar(65); }"));
    }

    #[test]
    fn if_without_else() {
        assert!(parse_succeeds("main() { if (x) x = 1; }"));
    }

    #[test]
    fn if_with_else() {
        assert!(parse_succeeds("main() { if (x) x = 1; else x = 2; }"));
    }

    #[test]
    fn nested_if_else_chain() {
        assert!(parse_succeeds(
            "main() { if (a) if (b) x = 1; else x = 2; else x = 3; }",
        ));
    }

    #[test]
    fn while_loop() {
        assert!(parse_succeeds("main() { while (x) x--; }"));
    }

    #[test]
    fn nested_while() {
        assert!(parse_succeeds("main() { while (a) while (b) x--; }"));
    }

    #[test]
    fn label_and_goto() {
        assert!(parse_succeeds("main() { L: x = 1; goto L; }"));
    }

    #[test]
    fn return_with_value() {
        assert!(parse_succeeds("main() { return (42); }"));
    }

    #[test]
    fn return_without_value() {
        assert!(parse_succeeds("main() { return; }"));
    }

    #[test]
    fn null_statement() {
        assert!(parse_succeeds("main() { ; }"));
    }

    #[test]
    fn switch_case_default() {
        assert!(parse_succeeds(
            "main() { switch x { case 1: x = 1; case 2: x = 2; default: x = 0; } }",
        ));
    }

    // ---- compound / edge-case assignments ----

    #[test]
    fn compound_assign_plus() {
        assert!(parse_succeeds("main() { x =+ 1; }"));
    }

    #[test]
    fn compound_assign_all() {
        assert!(parse_succeeds("main() { a =+ 1; b =- 2; c =* 3; d =/ 4; e =% 5; f =& 6; }"));
    }

    // ---- stress: deep nesting that could blow the stack ----

    #[test]
    fn deep_if_nesting() {
        let depth = 100;
        let mut src = String::from("main() {\n");
        for _ in 0..depth {
            src.push_str("if (x) {\n");
        }
        src.push_str("x = 1;\n");
        for _ in 0..depth {
            src.push_str("}\n");
        }
        src.push_str("}\n");
        assert!(parse_succeeds(&src));
    }

    #[test]
    fn deep_while_nesting() {
        let depth = 100;
        let mut src = String::from("main() {\n");
        for _ in 0..depth {
            src.push_str("while (x) {\n");
        }
        src.push_str("x--;\n");
        for _ in 0..depth {
            src.push_str("}\n");
        }
        src.push_str("}\n");
        assert!(parse_succeeds(&src));
    }

    #[test]
    fn deep_binary_expression_chain() {
        let count = 300;
        let expr = (0..count).map(|_| "x").collect::<Vec<_>>().join(" + ");
        let src = format!("main() {{ {}; }}", expr);
        assert!(parse_succeeds(&src));
    }

    #[test]
    fn deep_grouped_expression() {
        let depth = 200;
        let open: String = "(".repeat(depth);
        let close: String = ")".repeat(depth);
        let src = format!("main() {{ {}x{}; }}", open, close);
        assert!(parse_succeeds(&src));
    }

    #[test]
    fn many_function_calls_chained() {
        let depth = 100;
        let open: String = "f(".repeat(depth);
        let close: String = ")".repeat(depth);
        let src = format!("main() {{ {}x{}; }}", open, close);
        assert!(parse_succeeds(&src));
    }

    #[test]
    fn many_auto_declarations() {
        let names: Vec<String> = (0..200).map(|i| format!("v{}", i)).collect();
        let src = format!("main() {{ auto {}; v0 = 1; }}", names.join(", "));
        assert!(parse_succeeds(&src));
    }

    #[test]
    fn many_globals_in_sequence() {
        let mut src = String::new();
        for i in 0..200 {
            src.push_str(&format!("g{};", i));
        }
        src.push_str("main() ;");
        assert_eq!(parse_item_count(&src), 201);
    }

    #[test]
    fn many_global_array_initializers() {
        let init: Vec<String> = (0..100).map(|i| i.to_string()).collect();
        let src = format!("arr[100] {};\nmain() ;", init.join(","));
        assert_eq!(parse_item_count(&src), 2);
    }

    #[test]
    fn ternary_chain_deep() {
        let depth = 50;
        let mut expr = String::from("0");
        for i in (0..depth).rev() {
            expr = format!("(x ? {} : {})", i, expr);
        }
        let src = format!("main() {{ auto x; x = {}; }}", expr);
        assert!(parse_succeeds(&src));
    }

    #[test]
    fn multiple_functions() {
        let mut src = String::new();
        for i in 0..50 {
            src.push_str(&format!("f{}() {{ auto x; x = {}; }}\n", i, i));
        }
        assert_eq!(parse_item_count(&src), 50);
    }

    // ---- error cases ----

    #[test]
    fn rvalue_assign_is_error() {
        assert!(parse_fails("main() { 42 = x; }"));
    }
}

// ===========================================================================

#[cfg(test)]
mod e2e_tests {
    //! End-to-end: lex → parse, testing that the pipeline handles edge-case
    //! source inputs without panics or excessive resource usage.

    use b::lexer::scanner::Scanner;
    use b::lexer::token::*;
    use b::parser::Parser;

    fn lex_and_parse(src: &str) -> bool {
        let tokens: Result<Vec<_>, _> = Scanner::new(src).collect();
        let Ok(mut tokens) = tokens else { return false };
        tokens.push(SpannedToken {
            token: Token::EOF,
            span: b::common::span::Span::default(),
        });
        let mut parser = Parser::new(&tokens);
        parser.parse_program().is_ok()
    }

    #[test]
    fn e2e_stress_many_functions() {
        let mut src = String::new();
        for i in 0..500 {
            src.push_str(&format!(
                "f{}() {{ auto a, b; a = {}; b = a + 1; if (a == b) a =+ b; }}\n",
                i, i
            ));
        }
        assert!(lex_and_parse(&src));
    }

    #[test]
    fn e2e_stress_large_array_initializers() {
        let init: Vec<String> = (0..500).map(|i| i.to_string()).collect();
        let src = format!("arr[500] {};\nmain() {{ auto x; x = arr[0]; }}", init.join(","));
        assert!(lex_and_parse(&src));
    }

    #[test]
    fn e2e_string_heavy() {
        let mut src = String::from("main() { auto x;\n");
        for _ in 0..200 {
            src.push_str(r#"x = "hello*nworld*ttab";"#);
            src.push('\n');
        }
        src.push_str("}\n");
        assert!(lex_and_parse(&src));
    }

    #[test]
    fn e2e_comment_heavy() {
        let mut src = String::new();
        for i in 0..500 {
            src.push_str(&format!("/* comment {} */\n", i));
        }
        src.push_str("main() { auto x; x = 1; }\n");
        assert!(lex_and_parse(&src));
    }

    #[test]
    fn e2e_deep_nesting_combo() {
        let depth = 50;
        let mut src = String::from("main() { auto x;\n");
        for _ in 0..depth {
            src.push_str("if (x) { while (x) {\n");
        }
        src.push_str("x--;\n");
        for _ in 0..depth {
            src.push_str("} }\n");
        }
        src.push_str("}\n");
        assert!(lex_and_parse(&src));
    }

    #[test]
    fn e2e_mixed_globals_and_functions() {
        let mut src = String::new();
        for i in 0..100 {
            src.push_str(&format!("g{}[2] {},{};\n", i, i, i + 1));
            src.push_str(&format!("f{}() {{ auto x; x = g{}[0]; }}\n", i, i));
        }
        assert!(lex_and_parse(&src));
    }

    #[test]
    fn e2e_only_semicolons() {
        // A program of just global variable declarations with no names should fail.
        let src = ";;;";
        // This should fail or produce an empty program. Either way, no panic.
        let _ = lex_and_parse(&src);
    }

    #[test]
    fn e2e_identifier_at_i64_boundary() {
        // Using i64::MAX as a literal.
        let src = format!("main() {{ auto x; x = {}; }}", i64::MAX);
        // Might succeed or fail depending on number parsing, but must not panic.
        let _ = lex_and_parse(&src);
    }

    #[test]
    fn e2e_all_operators() {
        let src = r#"
            main() {
                auto a, b, c;
                a = 1;
                b = 2;
                c = a + b;
                c = a - b;
                c = a * b;
                c = a / b;
                c = a % b;
                c = a & b;
                c = a | b;
                c = a ^ b;
                c = a << b;
                c = a >> b;
                c = !a;
                c = ~a;
                c = a == b;
                c = a < b;
                c = a > b;
                c = a <= b;
                c = a >= b;
                a++;
                b--;
                ++a;
                --b;
                a =+ b;
                a =- b;
                a =* b;
                a =/ b;
                a =% b;
                a =& b;
                c = a ? b : 0;
            }
        "#;
        assert!(lex_and_parse(src));
    }
}
