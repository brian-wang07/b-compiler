#![allow(dead_code)]
use super::*;



//Abstract Visitor traits
pub trait ExprVisitor<T> {
  fn visit_assign(&mut self, lvalue: &Expr, operator: &Token, value: &Expr) -> T;
  fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
  fn visit_call(&mut self, callee: &Expr, arguments: &[Box<Expr>]) -> T;
  fn visit_grouping(&mut self, expression: &Expr) -> T;
  fn visit_literal(&mut self, value: &Token ) -> T;
  fn visit_unary(&mut self, operator: &Token, right: &Expr) -> T;
  fn visit_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
  fn visit_variable(&mut self, name: &Token) -> T;
  fn visit_get(&mut self, target: &Expr, index: &Expr) -> T;
  fn visit_ternary(&mut self, condition: &Expr, then_branch: &Expr, else_branch: &Expr) -> T;
  fn visit_postfix(&mut self, left: &Expr, operator: &Token) -> T;
}

pub trait StmtVisitor<T> {
  fn visit_block(&mut self, statements: &[Stmt]) -> T;
  fn visit_auto(&mut self, declarations: &Vec<AutoDecl>) -> T;
  fn visit_extrn(&mut self, names: &Vec<Token>) -> T;
  fn visit_expression(&mut self, expression: &Expr) -> T;
  fn visit_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: Option<&Stmt>) -> T;
  fn visit_var(&mut self, name: &Token, initializer: &Expr) -> T;
  fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> T;
  fn visit_switch(&mut self, condition: &Expr, body: &Stmt) -> T;
  fn visit_case(&mut self, value: &Token) -> T;
  fn visit_default(&mut self) -> T;
  fn visit_label(&mut self, name: &Token, body: &Stmt) -> T;
  fn visit_goto(&mut self, expression: &Expr) -> T;
  fn visit_return(&mut self, value: Option<&Expr>) -> T;
  fn visit_null(&mut self) -> T;
}

//Abstract interface: Handle recursion logic here. Matching expr allows for exhaustive checks, and improve runtime performance through static dispatch (no vtable lookup).
pub fn walk_expr<T, V: ExprVisitor<T>>(visitor: &mut V, expr: &Expr) -> T {
  //T is return type, V is concrete visitor walking through ast
  //generic type T is used so that visitor can be used for different passes.

  match expr {

    Expr::Assign { lvalue, operator, value } => {
      visitor.visit_assign(lvalue, operator, value)
    }

    Expr::Binary{ left, operator, right} => {
      visitor.visit_binary(left, operator, right)
    }

    Expr::Call { callee, arguments} => {
      visitor.visit_call(callee, arguments)
    }

    Expr::Grouping { expression } => {
      visitor.visit_grouping(expression)
    }

    Expr::Literal { value } => {
      visitor.visit_literal(value)
    }

    Expr::Unary { operator, right } => {
      visitor.visit_unary(operator, right)
    }

    Expr::Bitwise { left, operator, right } => {
      visitor.visit_logical(left, operator, right)
    }

    Expr::Variable { name } => {
      visitor.visit_variable(name)
    }

    Expr::Get { target, index } => {
      visitor.visit_get(target, index)
    }

    Expr::Ternary { condition, then_branch, else_branch } => {
      visitor.visit_ternary(condition, then_branch, else_branch)
    }

    Expr::Postfix { left, operator } => {
      visitor.visit_postfix(left, operator)
    }

  }

}

pub fn walk_stmt<T, V: StmtVisitor<T>>(visitor: &mut V, stmt: &Stmt) -> T {

  match stmt {

    Stmt::Block { statements } => {
      visitor.visit_block(statements)
    }

    Stmt::Auto { declarations } => {
      visitor.visit_auto(declarations)
    }

    Stmt::Extrn { names } => {
      visitor.visit_extrn(names)
    }

    Stmt::Expression { expression } => {
      visitor.visit_expression(expression)
    }

    Stmt::If { condition, then_branch, else_branch } => {
      visitor.visit_if(condition, then_branch, else_branch.as_deref())
    }

    Stmt::Var { name, initializer } => {
      visitor.visit_var(name, initializer)
    }

    Stmt::While { condition, body } => {
      visitor.visit_while(condition, body)
    }

    Stmt::Switch { condition, body } => {
      visitor.visit_switch(condition, body)
    }

    Stmt::Case { value } => {
      visitor.visit_case(value)
    }

    Stmt::Default => {
      visitor.visit_default()
    }

    Stmt::Label { name, body } => {
      visitor.visit_label(name, body)
    }

    Stmt::Goto { expression } => {
      visitor.visit_goto(expression)
    }

    Stmt::Return { value } => {
      visitor.visit_return(value.as_deref())
    }

    Stmt::Null => {
      visitor.visit_null()
    }
  

  }

  
}
