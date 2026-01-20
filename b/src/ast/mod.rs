use crate::lexer::token::{Token};
pub mod visitor;

//rvalue is any temporary value, doesnt have position in memory
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Assign { lvalue: Box<Expr>, operator: Token, value: Box<Expr> }, //variable assignment
  Binary { left: Box<Expr>, operator: Token, right: Box<Expr> }, //binary op
  Call { callee: Box<Expr>, arguments: Vec<Box<Expr>> }, //function call
  Grouping { expression: Box<Expr> }, //brackets
  Literal { value: Token }, 
  Unary { operator: Token, right: Box<Expr> }, //unary op
  Logical { left: Box<Expr>, operator: Token, right: Box<Expr>}, //bitwise
  Variable { name: Token }, //variable use (symbol table)
  Get { target: Box<Expr>, index: Box<Expr>}, //array index
  Ternary { condition: Box<Expr>, then_branch: Box<Expr>, else_branch: Box<Expr>},
  Postfix { left: Box<Expr>, operator: Token} //post inc/dec

}


//variables must be allocated before usage; auto a = 5 is invalid.
//auto and extrn must be first statement in a block
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
  Block { statements: Vec<Box<Stmt>> },
  Auto { declarations: Vec<(Token, Option<Token>)> }, // auto a, b, c[10]; option = some => size of arr
  Extrn { names: Vec<Token> }, //namespace op, must be declared with auto else raise compile error
  Expression { expression: Box<Expr> },
  If { condition: Box<Stmt>, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> }, //else branch can fall through
  Var { name: Token, initializer: Expr},
  While { condition: Box<Expr>, body: Box<Stmt> },
  Switch { condition: Box<Expr>, body: Box<Stmt> },
  Case { value: Token },
  Default,
  Label { name: Token, body: Box<Stmt> },
  Goto { expression: Box<Expr> },
  Return { value: Option<Box<Expr>> }, //return und if Option<T> = None
  Null,
  
}