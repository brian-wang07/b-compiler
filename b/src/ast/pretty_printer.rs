use crate::ast::visitor::{ExprVisitor, StmtVisitor, ItemVisitor, walk_expr, walk_stmt, walk_item};
use crate::ast::{Expr, Stmt, AutoDecl, Program, Item, GlobalDecl};
use crate::lexer::token::SpannedToken;

pub struct AstPrinter {
  indent: usize,
}

impl AstPrinter {
  pub fn new() -> Self {
    Self { indent: 0 }
  }

  pub fn print_expr(&mut self, expr: &Expr) -> String {
    walk_expr(self, expr)
  }

  pub fn print_stmt(&mut self, stmt: &Stmt) -> String {
    walk_stmt(self, stmt)
  }

  pub fn print_item(&mut self, item: &Item) -> String {
    walk_item(self, item)
  }

  pub fn print_program(&mut self, program: &Program) -> String {
    let mut result = String::new();
    for (i, item) in program.items.iter().enumerate() {
      if i > 0 {
        result.push_str("\n\n");
      }
      result.push_str(&self.print_item(item));
    }
    result
  }

  fn indent_str(&self) -> String {
    "  ".repeat(self.indent)
  }

  pub fn parenthesize(&mut self, op: &str, exprs: &[&Expr]) -> String {
    //format in reverse polish notation: 1 + 2 => (+ 1 2)
    let mut builder = String::new();
    builder.push('(');
    builder.push_str(op);
    for expr in exprs {
      builder.push(' ');
      builder.push_str(&self.print_expr(expr));
    }
    builder.push(')');
    builder
  }
}

impl StmtVisitor<String> for AstPrinter {

  fn visit_auto(&mut self, declarations: &[AutoDecl]) -> String {
    let parts: Vec<String> = declarations.iter().map(|decl| {
      match &decl.size {
        Some(size) => format!("{}[{}]", decl.name.token, size.token),
        None => format!("{}", decl.name.token),
      }
    }).collect();
    format!("{}(auto {})", self.indent_str(), parts.join(" "))
  }

  fn visit_block(&mut self, statements: &[Stmt]) -> String {
    let mut result = format!("{}(block\n", self.indent_str());
    //let mut result = String::from("(block\n");
    self.indent += 1;
    for stmt in statements {
      result.push_str(&self.print_stmt(stmt));
      result.push('\n');
    }
    self.indent -= 1;
    result.push_str(&self.indent_str());
    result.push(')');
    result
  }

  fn visit_case(&mut self, value: &SpannedToken, body: &Stmt) -> String {
    let mut result = format!("{}(case {}\n", self.indent_str(), value.token);
    self.indent += 1;
    result.push_str(&self.print_stmt(body));
    result.push('\n');
    self.indent -= 1;
    result.push_str(&self.indent_str());
    result.push(')');
    result
  }

  fn visit_default(&mut self, body: &Stmt) -> String {
    let mut result = format!("{}(default\n", self.indent_str());
    self.indent += 1;
    result.push_str(&self.print_stmt(body));
    result.push('\n');
    self.indent -= 1;
    result.push_str(&self.indent_str());
    result.push(')');
    result
  }

  fn visit_expression(&mut self, expression: &Expr) -> String {
    format!("{}(expr {})", self.indent_str(), self.print_expr(expression))
  }

  fn visit_extrn(&mut self, names: &[SpannedToken]) -> String {
    let names_str: Vec<String> = names.iter().map(|n| format!("{}", n.token)).collect();
    format!("{}(extrn {})", self.indent_str(), names_str.join(" "))
  }

  fn visit_goto(&mut self, expression: &Expr) -> String {
    format!("{}(goto {})", self.indent_str(), self.print_expr(expression))
  }

  fn visit_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: Option<&Stmt>) -> String {
    let mut result = format!("{}(if {}\n", self.indent_str(), self.print_expr(condition));
    self.indent += 1;
    result.push_str(&self.print_stmt(then_branch));
    if let Some(else_stmt) = else_branch {
      result.push('\n');
      result.push_str(&format!("{}(else\n", self.indent_str()));
      self.indent += 1;
      result.push_str(&self.print_stmt(else_stmt));
      result.push('\n');
      self.indent -= 1;
      result.push_str(&self.indent_str());
      result.push(')');
    }
    result.push('\n');
    self.indent -= 1;
    result.push_str(&self.indent_str());
    result.push(')');
    result
  }

  fn visit_label(&mut self, name: &SpannedToken, body: &Stmt) -> String {
    let mut result = format!("{}(label {}\n", self.indent_str(), name.token);
    self.indent += 1;
    result.push_str(&self.print_stmt(body));
    result.push('\n');
    self.indent -= 1;
    result.push_str(&self.indent_str());
    result.push(')');
    result
  }

  fn visit_null(&mut self) -> String {
    format!("{}(null)", self.indent_str())
  }

  fn visit_return(&mut self, value: Option<&Expr>) -> String {
    match value {
      Some(expr) => format!("{}(return {})", self.indent_str(), self.print_expr(expr)),
      None => format!("{}(return)", self.indent_str()),
    }
  }

  fn visit_switch(&mut self, condition: &Expr, cases: &[Stmt]) -> String {
    let mut result = format!("{}(switch {}\n", self.indent_str(), self.print_expr(condition));
    self.indent += 1;
    for case in cases {
      result.push_str(&self.print_stmt(case));
      result.push('\n');
    }
    self.indent -= 1;
    result.push_str(&self.indent_str());
    result.push(')');
    result
  }

  fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> String {
    let mut result = format!("{}(while {}\n", self.indent_str(), self.print_expr(condition));
    self.indent += 1;
    result.push_str(&self.print_stmt(body));
    result.push('\n');
    self.indent -= 1;
    result.push_str(&self.indent_str());
    result.push(')');
    result
  }
}

impl ItemVisitor<String> for AstPrinter {

  fn visit_function(&mut self, name: &SpannedToken, params: &[SpannedToken], body: &Stmt) -> String {
    let params_str: Vec<String> = params.iter().map(|p| format!("{}", p.token)).collect();
    let mut result = format!("(fn {} ({})\n", name.token, params_str.join(" "));
    self.indent += 1;
    result.push_str(&self.print_stmt(body));
    result.push('\n');
    self.indent -= 1;
    result.push(')');
    result
  }

  fn visit_global(&mut self, decls: &[GlobalDecl]) -> String {
    let parts: Vec<String> = decls.iter().map(|decl| {
      let mut s = format!("{}", decl.name.token);
      if let Some(size) = decl.size {
        s.push_str(&format!("[{}]", size.token));
      }
      if let Some(ref inits) = decl.initializer {
        let init_strs: Vec<String> = inits.iter().map(|i| format!("{}", i.token)).collect();
        s.push_str(&format!(" {}", init_strs.join(" ")));
      }
      s
    }).collect();
    format!("(global {})", parts.join(" "))
  }
}

impl ExprVisitor<String> for AstPrinter {

  fn visit_binary(&mut self, left: &Expr, operator: &SpannedToken, right: &Expr) -> String {
    self.parenthesize(&format!("{}", operator.token), &[left, right])
  }
  
  fn visit_assign(&mut self, lvalue: &Expr, operator: &SpannedToken, value: &Expr) -> String {
    self.parenthesize(&format!("{}", operator.token), &[lvalue, value])
  }

  fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> String {
    let mut builder = String::from("(call ");
    builder.push_str(&self.print_expr(callee));
    for arg in arguments {
      builder.push(' ');
      builder.push_str(&self.print_expr(arg));
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

  fn visit_literal(&mut self, value: &SpannedToken ) -> String {
    format!("{}", value.token)
  }

  fn visit_logical(&mut self, left: &Expr, operator: &SpannedToken, right: &Expr) -> String {
    self.parenthesize(&format!("{}", operator.token), &[left, right])
  }

  fn visit_postfix(&mut self, left: &Expr, operator: &SpannedToken) -> String {
      self.parenthesize(&format!("{} post", operator.token), &[left])
  }

  fn visit_ternary(&mut self, condition: &Expr, then_branch: &Expr, else_branch: &Expr) -> String {
      self.parenthesize("?:", &[condition, then_branch, else_branch])
  }

  fn visit_unary(&mut self, operator: &SpannedToken, right: &Expr) -> String {
      self.parenthesize(&format!("{}", operator.token), &[right])
  }

  fn visit_variable(&mut self, name: &SpannedToken) -> String {
      format!("{}", name.token)
  }
}