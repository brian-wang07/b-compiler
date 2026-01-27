use crate::common::span::Location;
pub mod scanner;
pub mod token;

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar(char, Location),
    UnterminatedString(Location),
    InvalidNumber(String, Location),
    UnterminatedComment(Location),
    UnterminatedChar(Location)
}
