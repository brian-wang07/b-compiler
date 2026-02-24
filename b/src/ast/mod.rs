use crate::lexer::token::{SpannedToken};
pub mod visitor;
pub mod pretty_printer;

#[derive(Debug, Clone, PartialEq)]
pub struct AutoDecl<'a> {
  pub name: SpannedToken<'a>,
  pub size: Option<SpannedToken<'a>>
}

//rvalue is any temporary value, doesnt have position in memory
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
  Assign { lvalue: Box<Expr<'a>>, operator: SpannedToken<'a>, value: Box<Expr<'a>> }, //variable assignment
  Binary { left: Box<Expr<'a>>, operator: SpannedToken<'a>, right: Box<Expr<'a>> }, //binary op
  Call { callee: Box<Expr<'a>>, arguments: Vec<Box<Expr<'a>>> }, //function call
  Grouping { expression: Box<Expr<'a>> }, //brackets
  Literal { value: SpannedToken<'a> }, 
  Unary { operator: SpannedToken<'a>, right: Box<Expr<'a>> }, //unary op
  Bitwise { left: Box<Expr<'a>>, operator: SpannedToken<'a>, right: Box<Expr<'a>>}, //bitwise
  Variable { name: SpannedToken<'a> }, //variable use (symbol table)
  Get { target: Box<Expr<'a>>, index: Box<Expr<'a>>}, //array index a[10]
  Ternary { condition: Box<Expr<'a>>, then_branch: Box<Expr<'a>>, else_branch: Box<Expr<'a>>},
  Postfix { left: Box<Expr<'a>>, operator: SpannedToken<'a>} //post inc/dec

}


//variables must be allocated before usage; auto a = 5 is invalid.
//auto and extrn must be first statement in a block
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'a> {
  Block { statements: Vec<Stmt<'a>> },
  Auto { declarations: Vec<AutoDecl<'a>> }, // auto a, b, c[10]; option = some => size of arr
  Extrn { names: Vec<SpannedToken<'a>> }, //namespace op, must be declared with auto else raise compile error
  Expression { expression: Box<Expr<'a>> },
  If { condition: Box<Expr<'a>>, then_branch: Box<Stmt<'a>>, else_branch: Option<Box<Stmt<'a>>> }, //else branch can fall through
  //Var { name: SpannedToken<'a>, initializer: Box<Expr<'a>>},
  While { condition: Box<Expr<'a>>, body: Box<Stmt<'a>> },
  Switch { condition: Box<Expr<'a>>, cases: Vec<Stmt<'a>> },
  Case { value: SpannedToken<'a>, body: Box<Stmt<'a>> },
  Default,
  Label { name: SpannedToken<'a>, body: Box<Stmt<'a>> },
  Goto { expression: Box<Expr<'a>> },
  Return { value: Option<Box<Expr<'a>>> }, //return und if Option<T> = None
  Null,
  
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program<'a> {
  pub top_level: Vec<TopLevel<'a>>,
}

//top only valid top levels are functions and global declarations
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevel<'a> {
  Function(Function<'a>),
  Global(Vec<GlobalDecl<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function<'a>{
  pub name: &'a SpannedToken<'a>,
  pub params: Vec<&'a SpannedToken<'a>>,
  pub body: Stmt<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDecl<'a> {
  pub name: &'a SpannedToken<'a>,
  pub size: Option<&'a SpannedToken<'a>>,
  pub initializer: Option<Vec<&'a SpannedToken<'a>>>,
}