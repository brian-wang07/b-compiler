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
    //semicolon is NOT consumed!!!!!
    let mut is_err = false;
    //array
    match self.peek().token {
      Token::Delimiter(Delimiter::LBrack) => {
        let mut array_size = None;
        let mut initializer_list = Vec::new();
        //array declaration
        //consume [
        self.advance();
        //check for array size
        match self.peek().token {
          //specified array size
          Token::Integer(..) => {
            let mut array_size = Some(self.advance());
          },

          Token::Delimiter(Delimiter::RBrack) => {
            let mut array_size: Option<&SpannedToken<'_>> = None;
          },
          _ => is_err = false,
        }

        if is_err {return Err(ParseError::UnknownToken(self.peek().clone())); }

        self.expect(&Token::Delimiter(Delimiter::RBrack))?;

        //check for array initialization
        match self.peek().token {
          //Array initializers can only be constant rvalues (no ident)
          Token::Integer(..) | Token::CharLiteral(..) | Token::StringLiteral(..) => {
            if array_size == None {
              return Err(ParseError::UnspecifiedArraySizeInitialization(self.peek().clone()));
            }
            //parse initializers
            loop {
              initializer_list.push(self.advance());
              match self.peek().token {

                Token::Delimiter(Delimiter::Comma) => {
                  self.advance();
                  match self.peek().token {
                    //if the token after the comma is an ident, then we have finished initializing.
                    Token::Identifier(..) => break,
                    Token::Integer(..) | Token::CharLiteral(..) | Token::StringLiteral(..) => continue,
                    _ => is_err = true,
                  }

                  if is_err {
                    return Err(ParseError::UnknownToken(self.peek().clone()));
                  }
                }

                Token::Delimiter(Delimiter::Semicolon) => break,
                _ => is_err = true,
              }

              if is_err {
                return Err(ParseError::UnknownToken(self.peek().clone()));
              }
            }
            Ok(GlobalDecl {
              name: name,
              size: array_size,
              initializer: Some(initializer_list),
            })
          }

          Token::Delimiter(Delimiter::Semicolon) | Token::Delimiter(Delimiter::Comma) => {
            //no initializers
            Ok(GlobalDecl {
              name: name,
              size: array_size,
              initializer: None
            })
          }

          _ => {
            return Err(ParseError::UnknownToken(self.peek().clone()));
          }
        }
      }

      Token::Integer(..) | Token::CharLiteral(..) | Token::StringLiteral(..) => {
        //scalar with initializer
        let mut initializer = Vec::new();
        initializer.push(self.advance());
        Ok(GlobalDecl {
          name: name,
          size: None,
          initializer: Some(initializer),
        })
      }

      Token::Delimiter(Delimiter::Semicolon) | Token::Delimiter(Delimiter::Comma) => {
        //scalar with no initializer
        Ok(GlobalDecl {
          name: name,
          size: None,
          initializer: None
        })
      }

      _ => {
        return Err(ParseError::UnknownToken(self.peek().clone()));
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
  //do top level extrn later im too lazy but it wont be hard (add an is_extrn field to global node)

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
          size: None,
          initializer: None,
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
        return Err(ParseError::AutoRedecl(self.peek().clone()));
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
    let mut decls = Vec::new();
    decls.push(self.advance().clone());
    while self.peek().token == Token::Delimiter(Delimiter::Comma) {
      self.advance();
      decls.push(self.advance().clone());
    }

    self.expect(&Token::Delimiter(Delimiter::Semicolon));
    Ok(Stmt::Extrn {
      names: decls,
    })
    
  }

  pub fn parse_if(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {
    //Syntax: if(condition) then_statement else_statement (optional)
    self.expect(&Token::Delimiter(Delimiter::LParen))?;
    let condition = self.parse_expression(0)?;
    self.expect(&Token::Delimiter(Delimiter::RParen))?;
    let then_branch = self.parse_statement()?;
    if self.peek().token == Token::Keyword(Keyword::Else) {
      let else_branch = self.parse_statement()?;
      return Ok(Stmt::If {
        condition: Box::new(condition),
        then_branch: Box::new(then_branch),
        else_branch: Some(Box::new(else_branch)),
      });
    }
    Ok(Stmt::If {
      condition: Box::new(condition),
      then_branch: Box::new(then_branch),
      else_branch: None,
    })


  }

  pub fn parse_while(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {
    //Syntax: while(condition) body
    self.expect(&Token::Delimiter(Delimiter::LParen))?;
    let condition = self.parse_expression(0)?;
    let body = self.parse_statement()?;
    Ok(Stmt::While {
      condition: Box::new(condition),
      body: Box::new(body),
    })
  }

  pub fn parse_switch(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {
    //Syntax: switch expr {cases}
    let mut is_err = false;
    let mut cases = Vec::new();
    let condition = self.parse_expression(0)?;
    //dispatch body through parse_case instead of parse_statement
    self.expect(&Token::Delimiter(Delimiter::LBrace))?;
    loop {
      match self.peek().token {
        Token::Keyword(Keyword::Case) => {
          //consume Case keyword, parse body
          self.advance();
          let case = self.parse_case()?;
          cases.push(case);
        }
        Token::Delimiter(Delimiter::RBrack) => break,
        _ => is_err = {is_err = true; break;}

      }
    }
    if is_err {
      return Err(ParseError::UnknownToken(self.peek().clone()))
    }
    self.advance();
    Ok(Stmt::Switch {
      condition: Box::new(condition),
      cases: cases,
    })


  }

  pub fn parse_case(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {
    let value = self.advance();
    let body = self.parse_statement()?;
    Ok(Stmt::Case {
      value: value.clone(),
      body: Box::new(body),
    })
  }

  pub fn parse_goto(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {
    let expr = self.parse_expression(0)?;
    Ok(Stmt::Goto {
      expression: Box::new(expr),
    })
  }

  pub fn parse_return(&mut self) -> Result<Stmt<'a>, ParseError<'a>> {
    if self.peek().token == Token::Delimiter(Delimiter::Semicolon) {
      return Ok(Stmt::Return {
        value: None,
      });
    }
    let expr = self.parse_expression(0)?;
    Ok(Stmt::Return {
      value: Some(Box::new(expr)),
    })
  }


}