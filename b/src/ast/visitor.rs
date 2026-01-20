#![allow(dead_code)]
use super::*;

trait ExprVisitor<T> {
  fn visit_assign(&mut self, lvalue: &Expr, operator: Token, value: &Expr) -> T;
  fn visit_binary(&mut self, left: &Expr, operator: Token, right: &Expr) -> T;


}


