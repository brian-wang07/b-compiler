use super::{Parser, ParseError, Precedence};
use crate::ast::Expr;
use crate::lexer::token::*;


//expression parsing: pratt parser
impl <'a> Parser<'a> {


  fn is_lvalue(&self, expr: &Expr<'a>) -> bool {
    //lvalues are only variables, array indicies, and dereferences
    if matches!(expr, Expr::Variable {..}| Expr::Get {..} | Expr::Unary { operator: SpannedToken{ token: Token::Operator(Operator::Star), .. }, .. }) {
      return true;
    }
    false
  }

  ///Null denotation: Parse tokens that start an expression. Includes
  /// Variable identifiers, literals, groupings, and prefix operators (preinc, 
  /// negation, etc.)
  fn nud(&mut self) -> Result<Expr<'a>, ParseError<'a>> {

    let t = self.advance(); //need to clone :( transfer reference ownership from parser to t
    //clone in return, so you dont lose lifetime annotation
    match &t.token {

      Token::Integer(_) | Token::CharLiteral(_) |
      Token::StringLiteral(_) => Ok(Expr::Literal{ value: t.clone() }),

      Token::Identifier(_) => Ok(Expr::Variable{ name: t.clone() }),

      Token::Operator(Operator::Plus) | Token::Operator(Operator::Minus) | 
      Token::Operator(Operator::Bang) | Token::Operator(Operator::Tilde) |
      Token::Operator(Operator::Inc) | Token::Operator(Operator::Dec) |
      Token::Operator(Operator::Amp) | Token::Operator(Operator::Star) => {
        let right = self.parse_expression(Precedence::Unary.bp().1); //rhbp
        Ok(Expr::Unary{ operator: t.clone(), right: Box::new(right.unwrap()) })
      },

      Token::Delimiter(Delimiter::LParen) => {
        let inner = self.parse_expression(Precedence::None.bp().1)?;
        self.expect(&Token::Delimiter(Delimiter::RParen))?;
        Ok(Expr::Grouping{ expression: Box::new(inner) })
      },



      _ => Err(ParseError::UnknownToken(t.clone())),
            
      }

  }


  ///Left Denotation: After parsing a null, led() is called. All unfix, postfix, and assignment operators are handled here.
  fn led(&mut self, left: Expr<'a>) -> Result<Expr<'a>, ParseError<'a>> {
    //if we are here, literal tokens have already been parsed. for prefix, it goes nud -> parse_expr -> (grouping) -> parse_expr -> nud.
    //thus expect an operator token, or else we have something like ab + c.
    let op = self.advance().clone(); //op has ownership (cringe ahh clone again, but too lazy to change advance)
    
    match op.token {
      Token::Integer(_) | Token::StringLiteral(_) | 
      Token::CharLiteral(_) | Token::Identifier(_) => Err(ParseError::UnknownToken(op)),

      Token::Operator(Operator::Inc) | Token::Operator(Operator::Dec) => Ok(Expr::Postfix{left: Box::new(left), operator: op}),

      Token::Delimiter(t) => {
        match t {

          Delimiter::LParen => {
            let mut args = Vec::new();
            if !(self.peek().token == Token::Delimiter(Delimiter::RParen)) {
              loop {
                let arg = self.parse_expression(Precedence::None.bp().1)?;
                args.push(Box::new(arg));
                
                if self.peek().token == Token::Delimiter(Delimiter::Comma) {
                  self.advance();
                }
                else {break;}
              }
            }

            self.expect(&Token::Delimiter(Delimiter::RParen))?;
            Ok(Expr::Call { callee: Box::new(left), arguments: args})
          }

          Delimiter::LBrack => {
            let index = self.parse_expression(Precedence::None.bp().1)?;
            self.expect(&Token::Delimiter(Delimiter::RBrack))?;
            Ok(Expr::Get { target: Box::new(left), index: Box::new(index)})
          },

          
          Delimiter::QMark => {
            let then_branch = self.parse_expression(Precedence::Ternary.bp().1)?;
            self.expect(&Token::Delimiter(Delimiter::Colon))?;
            let else_branch = self.parse_expression(Precedence::Ternary.bp().1)?;
            Ok(Expr::Ternary { condition: Box::new(left), then_branch: Box::new(then_branch), else_branch: Box::new(else_branch) })
          }
        
          _ => Err(ParseError::UnknownToken(op))

        }
      } 

      Token::Operator(t) => {
        let (_, rbp) = self.get_precedence(&op.token).bp();
        let right = self.parse_expression(rbp)?;
        match t {
          Operator::Plus | Operator::Minus | Operator::Star | Operator::Slash |
          Operator::Percent | Operator::Equal | Operator::NotEqual |
          Operator::Less | Operator::LessEq | Operator::Greater | 
          Operator::GreaterEq => Ok(Expr::Binary {left: Box::new(left), operator: op, right: Box::new(right)}),

          Operator::Amp | Operator::Bar | Operator::Caret | Operator::LShift |
          Operator::RShift | Operator::Tilde => Ok(Expr::Bitwise {left: Box::new(left), operator: op, right: Box::new(right)}),

          //assign: left must be lvalue
          Operator::Assign | Operator::AssignAmp | Operator::AssignMinus |
          Operator::AssignPercent | Operator::AssignPlus | Operator::AssignSlash |
          Operator::AssignStar => {
            if self.is_lvalue(&left) {
              return Ok(Expr::Assign{lvalue: Box::new(left), operator: op, value: Box::new(right)});
            }
            Err(ParseError::RValueAssign(op))
          }
          _ => Err(ParseError::UnknownToken(op)),

        }
      }

      _ => Err(ParseError::UnknownToken(op))

    }


  }
    ///pratt style parser: Top down, LL(1) recursive parser
    /// Precedence levels are defined in precedence.rs. Instead of a recursively defined formal grammar, the binding power
    /// of the operators define the order of precedence.     
  pub fn parse_expression(&mut self, min_prec: u8) -> Result<Expr<'a>, ParseError<'a>> {
    //parse left hand side (nud)
    let mut left = self.nud()?;
    //while left bp has higher precedence than min bp (right bp), parse left denotation
    while self.peek_precedence().bp().0 > min_prec {
      left = self.led(left)?;
    }

    Ok(left)
  }

}