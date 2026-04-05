use crate::lexer::token::{SpannedToken, Keyword, Operator};
pub mod visitor;
pub mod pretty_printer;


//rvalue is any temporary value, doesnt have position in memory
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
  Assign { lvalue: Box<Expr<'a>>, operator: Operator, value: Box<Expr<'a>> }, //variable assignment
  Binary { left: Box<Expr<'a>>, operator: Operator, right: Box<Expr<'a>> }, //binary op
  Call { callee: Box<Expr<'a>>, arguments: Vec<Expr<'a>> }, //function call
  Grouping { expression: Box<Expr<'a>> }, //brackets
  Literal { value: Lexeme<'a> }, 
  Unary { operator: Operator, right: Box<Expr<'a>> }, //unary op
  Variable { name: Lexeme<'a> }, //variable use (symbol table)
  Get { target: Box<Expr<'a>>, index: Box<Expr<'a>>}, //array index a[10]
  Ternary { condition: Box<Expr<'a>>, then_branch: Box<Expr<'a>>, else_branch: Box<Expr<'a>>},
  Postfix { left: Box<Expr<'a>>, operator: Operator} //post inc/dec

}


//variables must be allocated before usage; auto a = 5 is invalid.
//auto and extrn must be first statement in a block
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'a> {
  Block { statements: Vec<Stmt<'a>> },
  Auto { declarations: Vec<AutoDecl<'a>> }, // auto a, b, c[10]; option = some => size of arr
  Extrn { names: Vec<&'a SpannedToken<'a>> }, //namespace op, must be declared with auto else raise compile error
  Expression { expression: Box<Expr<'a>> },
  If { condition: Box<Expr<'a>>, then_branch: Box<Stmt<'a>>, else_branch: Option<Box<Stmt<'a>>> }, //else branch can fall through
  While { condition: Box<Expr<'a>>, body: Box<Stmt<'a>> },
  Switch { condition: Box<Expr<'a>>, cases: Vec<Stmt<'a>> },
  Case { value: SpannedToken<'a>, body: Box<Stmt<'a>> },
  Default {body: Box<Stmt<'a>>},
  Label { name: Lexeme<'a>, body: Box<Stmt<'a>> },
  Goto { expression: Box<Expr<'a>> },
  Return { value: Option<Box<Expr<'a>>> }, //return und if Option<T> = None
  Null,
  
}

#[derive(Debug, Clone, PartialEq)]
//parser entry point
pub struct Program<'a> {
  pub items: Vec<Item<'a>>,
}

//top only valid top levels are functions and global declarations
#[derive(Debug, Clone, PartialEq)]
pub enum Item<'a> {
  Function(Function<'a>),
  Global(Vec<GlobalDecl<'a>>),
}


//enum to hold literal types in AST - seperate from tokens so that 'src lifetime can be freed 
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lexeme<'a> {
  Int(i64),
  Str(&'a str),
  Char(i64),
  Ident(&'a str),
  Kw(Keyword), 
}




#[derive(Debug, Clone, PartialEq)]
pub struct Function<'a>{
  pub name: &'a SpannedToken<'a>,
  pub params: Vec<&'a SpannedToken<'a>>,
  pub body: Box<Stmt<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDecl<'a> {
  pub name: &'a SpannedToken<'a>,
  pub size: Option<&'a SpannedToken<'a>>,
  pub initializer: Option<Vec<&'a SpannedToken<'a>>>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct AutoDecl<'a> {
  pub name: &'a SpannedToken<'a>,
  pub size: Option<&'a SpannedToken<'a>>
}