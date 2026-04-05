use crate::common::span::Span;
pub mod scanner;
pub mod token;

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar(char, Span),
    UnterminatedString(Span),
    InvalidNumber(String, Span),
    UnterminatedComment(Span),
    UnterminatedChar(Span)
}
