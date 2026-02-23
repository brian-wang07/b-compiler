use crate::ast::*;
use crate::lexer::token::*;

use super::{Parser, ParseError, precedence};

//statement parsing: recursive descent
impl <'a> Parser<'a> {
 //enforce first line being auto; after parsing '{', check for auto keyword.
 //after parsing next line, set flag auto_decl = false. If an auto decl is seen again
 //in that block, raise error 

 //helper for parsing global declarations
  fn parse_decl(&mut self, name: &'a SpannedToken<'a>) -> Result<GlobalDecl<'a>, ParseError<'a>> {
    //array
    if self.peek().token == Token::Delimiter(Delimiter::LBrack) {
      self.advance();
      let size = self.advance();
      self.expect(&Token::Delimiter(Delimiter::RBrack))?;
      let mut initializers = Vec::new();
      let mut iserr = false;
      //check for initialization
      loop {
        match self.peek_next().clone() {
          Token::Identifier(..) => break,
          Token::Integer(..) | Token::StringLiteral(..) | Token::CharLiteral(..) => {
            self.advance();
            initializers.push(self.advance());
          }
          _ => {iserr = true; break;}
        }
      }
      self.advance();//consume comma
      if (iserr) {
        return Err(ParseError::UnknownToken(self.peek().clone()));
      }

      Ok(GlobalDecl {
        name: name,
        size: Some(size),
        initializer: Some(initializers),
      })
    }
    else {
      let mut initializer = Vec::new();
      let mut iserr = false;
      match self.peek_next().clone() {
        Token::Identifier(..) => {}
        Token::Integer(..) | Token::StringLiteral(..) | Token::CharLiteral(..) => {
          self.advance();
          initializer.push(self.advance());
        }
        _ => {iserr = true;}
      }
      self.advance();
      if (iserr) {
        return Err(ParseError::UnknownToken(self.peek().clone()));
      }
      if initializer.len() == 0 {
        return Ok(GlobalDecl {
          name: name,
          size: None,
          initializer: None 
        });
      }
      else {
        return Ok(GlobalDecl {
          name: name,
          size: None,
          initializer: Some(initializer)
        })
      }
    }
  }


  //3 Types of declarations: Auto, External, Internal.
  //Auto: Local variable that is freed when it goes out of scope. only valid in function scope
  //Extrn: External declaration that states that the variable has been declared somewhere else (resolved by linker). valid in global and function scope
  //External declarations are also done with no keyword in global scope. possible cases are:
  //x; allocates a variable x.
  //x a; allocates a variable x and initializes to a.
  //x[i]; allocates a vector x of size i (static size), initialize to 0.
  //x[i] a, b, c; allocatoes a vector x of size i and initializes first three elements to a, b, c. must be constants (numbers). initialize rest to 0. 
  //x a, b, c; allocates a vector x of size 3 and initializes to a, b, c.
  //note that these are all valid for auto declarations as well.
  //simple declaration is also valid within a function scope, which is internal declaration.

  pub fn parse_program(&mut self) -> Result<Program<'a>, ParseError<'a>> {
    let mut tops = Vec::new();
    while !self.is_at_end() {
      let top = self.parse_top_level()?;
      tops.push(top)
    }  
    Ok(Program{top_level: tops})
  }

  pub fn parse_top_level(&mut self) -> Result<TopLevel<'a>, ParseError<'a>> {
    //only functions and specific declarations (outlined above) are valid at the top level.
    let mut name = self.advance();
    match self.peek().token {
      //var name followed by ( is a function
      Token::Delimiter(Delimiter::LParen) => {
        let mut params = Vec::new();
        if !(self.peek().token == Token::Delimiter(Delimiter::LParen)) {
          loop {
            let arg = self.advance();
            params.push(arg);

            if self.peek().token == Token::Delimiter(Delimiter::Comma) {
              self.advance();
            }
            else {break;}
          }
        }

        self.expect(&Token::Delimiter(Delimiter::RParen))?;
        Ok(TopLevel::Function(Function {
          name: name,
          params: params,
          body: self.parse_statement()?,
        }))
      }

      //else, we have a global var decl, such as a, b[10], c;. 
      //one variable declared
      Token::Delimiter(Delimiter::Semicolon) => {
        Ok(TopLevel::Global(vec![GlobalDecl {
          name: name,
          size: None
        }]))
      }

      //multiple/array
      Token::Delimiter(Delimiter::LBrack) | Token::Delimiter(Delimiter::Comma) => {
        let mut decls = Vec::new();
        let mut decl = self.parse_decl(name)?;
        decls.push(decl);
        while self.peek().token == Token::Delimiter(Delimiter::Comma) {
          self.advance();
          name = self.advance();
          decl = self.parse_decl(name)?;
          decls.push(decl);
        }
        self.expect(&Token::Delimiter(Delimiter::Semicolon))?;
        Ok(TopLevel::Global(decls))
      }

      _ => Err(ParseError::UnknownToken(name.clone())),

    }

  }

  pub fn parse_statement(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {
    let t = self.advance();
    match t.token {
  
      Token::Delimiter(Delimiter::Semicolon) => Ok(Stmt::Null),
      Token::Delimiter(Delimiter::LBrace) => self.parse_block(),
      Token::Keyword(Keyword::Extrn) => self.parse_extrn(),
      Token::Keyword(Keyword::If) => self.parse_if(),
      Token::Keyword(Keyword::While) => self.parse_while(),
      Token::Keyword(Keyword::Switch) => self.parse_switch(),
      Token::Keyword(Keyword::Case) => self.parse_case(),
      Token::Keyword(Keyword::Goto) => self.parse_goto(),
      Token::Keyword(Keyword::Return) => self.parse_return(),


      _ => Err(ParseError::UnknownToken(t.clone())),
    }
  }

  pub fn parse_block(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {

    let mut valid_auto_decl = true;
    let mut statements = Vec::new();
    while self.peek().token != Token::Delimiter(Delimiter::RBrace) {

      //auto declared not as first statement
      if (self.peek().token == Token::Keyword(Keyword::Auto) && valid_auto_decl == false) {
        return Err(ParseError::AutoRedecl);
      }

      //valid auto
      if (self.peek().token == Token::Keyword(Keyword::Auto) && valid_auto_decl == true) {
        statements.push(self.parse_auto()?);
        valid_auto_decl = false;
        continue;
      }


      let statement = self.parse_statement()?;
      valid_auto_decl = false;
      statements.push(statement);
    }
    self.expect(&Token::Delimiter(Delimiter::RBrace));
    Ok(Stmt::Block {
      statements: statements,
    })

  }

  pub fn parse_auto(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {
    let mut decls = Vec::new();  
    while self.peek().token == Token::Delimiter(Delimiter::Comma) {
      let mut var = self.advance();
      if self.peek().token == Token::Delimiter(Delimiter::RBrack) {
        self.advance();
        let mut size = self.advance();
        self.expect(&Token::Delimiter(Delimiter::RBrack));
        decls.push(AutoDecl {
          name: var.clone(),
          size: Some(size.clone()),
        });
      }
      else {
        decls.push(AutoDecl {
          name: var.clone(),
          size: None,
        });
      }
      }
    self.expect(&Token::Delimiter(Delimiter::Semicolon));
    Ok(Stmt::Auto {
      declarations: decls,
    })


  }

  pub fn parse_extrn(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {
    
  }

  pub fn parse_if(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {

  }

  pub fn parse_while(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {

  }

  pub fn parse_switch(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {

  }

  pub fn parse_case(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {

  }

  pub fn parse_goto(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {

  }

  pub fn parse_return(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {

  }

}