#![allow(dead_code, unused)]
use crate::lexer::token::{SpannedToken,Token, Operator, Delimiter};
use crate::common::span::Span;
use crate::parser::precedence::Precedence;

pub mod precedence;
pub mod expr;
pub mod stmt;


#[derive(Debug)]
pub enum ParseError<'a> {
  UnexpectedToken(Expected<'a>),
  UnknownToken(SpannedToken<'a>),
  UnexpectedEOF,
  RValueAssign(SpannedToken<'a>),
  AutoRedecl(SpannedToken<'a>),
  UnspecifiedArraySizeInitialization(SpannedToken<'a>),
}

#[derive(Debug)]
pub struct Expected<'a> {
  expected: &'a Token<'a>,
  found: &'a SpannedToken<'a>,
}

#[derive(Debug)]
pub struct Parser<'a> {
  tokens: &'a [SpannedToken<'a>],
  position: usize,
}


impl<'a> Parser<'a> {

  //initizalize new parser
  pub fn new(tokens: &'a [SpannedToken<'a>]) -> Self {
    Parser {
      tokens,
      position: 0,
    }
  }

  //peek
  fn peek(&self) -> &'a SpannedToken<'a> {
    self.tokens.get(self.position)
      .unwrap_or_else(|| self.tokens.last().expect("Token Stream Empty"))
  }


  fn is_at_end(&self) -> bool {
    matches!(self.peek().token, Token::EOF)
  }

  //advance and return consumed token
  fn advance(&mut self) -> &'a SpannedToken<'a> {
    if !self.is_at_end() {
      self.position += 1 ;
    }

    self.tokens.get(self.position - 1)
      .expect("Advance called on empty token stream")
  }

  //match with expected; return bool and advance if matches
  fn match_token(&mut self, expected: &'a Token<'a>) -> bool {
    if self.peek().token == *expected {
      self.advance();
      return true;
    }
    false
  }

  //match with expected
  fn expect(&mut self, expected: &'a Token<'a>) -> Result<&'a SpannedToken<'a>, ParseError<'a>> {
    let tok = self.peek();
    if tok.token == *expected {
      return Ok(self.advance());
    }
    Err(ParseError::UnexpectedToken(Expected {
      expected: expected,
      found: tok
    }))
  }

  //cur span
  fn current_span(&self) -> Span {
    self.tokens.get(self.position)
      .map(|t| t.span.clone())
      .unwrap_or_default()
  }

  fn peek_next(&self) -> &Token<'a> {
    self.tokens.get(self.position + 1)
      .map(|t| &t.token)
      .unwrap_or(&Token::EOF)
  }




  //peek next token's precedence. Operations such as ++ and -- are set as postfix, as peek_precedence is only called after the LHS
  //has been parsed. prefix unary operators (such as -, !, ~, --, ++) are handled as nud(unary), and have no binding power.
  fn peek_precedence(&self) -> Precedence {
    self.get_precedence(&self.peek().token)
  }

  fn get_precedence(&self, tok: &Token) -> Precedence {
     match tok {
      Token::Operator(op) => match op {
        Operator::Assign | Operator::AssignPlus | Operator::AssignMinus |
        Operator::AssignStar | Operator::AssignSlash | Operator::AssignPercent |
        Operator::AssignAmp => Precedence::Assignment,
        Operator::Bar => Precedence::BitOr,
        Operator::Caret => Precedence::BitXor,
        Operator::Amp => Precedence::BitAnd,
        Operator::Equal | Operator::NotEqual => Precedence::Equality,
        Operator::Less | Operator::Greater | Operator::LessEq | Operator::GreaterEq => Precedence::Comparison,
        Operator::LShift | Operator::RShift => Precedence::Shift,
        Operator::Plus | Operator::Minus => Precedence::Addition,
        Operator::Star | Operator::Slash | Operator::Percent => Precedence::Multiplication,
        Operator::Inc | Operator::Dec => Precedence::Postfix,
        _ => Precedence::None,
      },

      Token::Delimiter(Delimiter::QMark) => Precedence::Ternary,
      Token::Delimiter(Delimiter::LParen) | Token::Delimiter(Delimiter::LBrack) => Precedence::Postfix,
      _ => Precedence::None,
    }
  }
}


