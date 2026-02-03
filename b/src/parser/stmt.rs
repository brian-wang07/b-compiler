use super::{Parser, ParseError, precedence};

//statement parsing: recursive descent
impl <'a> Parser<'a> {
 //enforce first line being auto; after parsing '{', check for auto keyword.
 //after parsing next line, set flag auto_decl = false. If an auto decl is seen again
 //in that block, raise error 
}