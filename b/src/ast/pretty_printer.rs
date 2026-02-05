use crate::ast::visitor::{ExprVisitor, walk_expr};
use crate::ast::Expr;
use crate::lexer::token::Token;

pub struct AstPrinter;

impl AstPrinter {
  pub fn new() -> Self {
    Self
  }

  pub fn print(&mut self, expr: &Expr) -> String {
    walk_expr(self, expr)
  }

  pub fn parenthesize(&mut self, op: &str, exprs: &[&Expr]) -> String {
    //format in reverse polish notation: 1 + 2 => (+ 1 2)
    let mut builder = String::new();
    builder.push('(');
    builder.push_str(op);
    for expr in exprs {
      builder.push(' ');
      builder.push_str(&self.print(expr));
    }
    builder.push(')');
    builder
  }
}


impl ExprVisitor<String> for AstPrinter {

  fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
    self.parenthesize(&format!("{}", operator), &[left, right])
  }
  
  fn visit_assign(&mut self, lvalue: &Expr, operator: &Token, value: &Expr) -> String {
    self.parenthesize(&format!("{}", operator), &[lvalue, value])
  }

  fn visit_call(&mut self, callee: &Expr, arguments: &[Box<Expr>]) -> String {
    let mut builder = String::from("(call ");
    builder.push_str(&self.print(callee));
    for arg in arguments {
      builder.push(' ');
      builder.push_str(&self.print(arg));
    }    
    builder.push(')');
    builder
  }

  fn visit_get(&mut self, target: &Expr, index: &Expr) -> String {
    self.parenthesize("get", &[target, index])
  }

  fn visit_grouping(&mut self, expression: &Expr) -> String {
    self.parenthesize("group", &[expression])
  }

  fn visit_literal(&mut self, value: &Token ) -> String {
    format!("{}", value)
  }

  fn visit_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
    self.parenthesize(&format!("{}", operator), &[left, right])
  }

  fn visit_postfix(&mut self, left: &Expr, operator: &Token) -> String {
      self.parenthesize(&format!("{} post", operator), &[left])
  }

  fn visit_ternary(&mut self, condition: &Expr, then_branch: &Expr, else_branch: &Expr) -> String {
      self.parenthesize("?:", &[condition, then_branch, else_branch])
  }

  fn visit_unary(&mut self, operator: &Token, right: &Expr) -> String {
      self.parenthesize(&format!("{}", operator), &[right])
  }

  fn visit_variable(&mut self, name: &Token) -> String {
      format!("{}", name)
  }
}