use crate::error::SaturdayResult;
use crate::expr::{
  AssignExpr, BinaryExpr, CallExpr, Expr, ExprVisitor, GroupingExpr, LiteralExpr, LogicalExpr,
  UnaryExpr, VariableExpr,
};
use crate::interpreter::Interpreter;
use crate::stmt::{
  BlockStmt, BreakStmt, DefStmt, ExpressionStmt, FunctionStmt, IfStmt, PrintStmt, ReturnStmt, Stmt,
  StmtVisitor, WhileStmt,
};
use crate::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub struct Resolver<'a> {
  interpreter: &'a Interpreter,
  scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
  had_error: RefCell<bool>,
  current_function: RefCell<FunctionType>,
  in_while: RefCell<bool>,
}

#[derive(PartialEq)]
enum FunctionType {
  None,
  Function,
}

impl<'a> Resolver<'a> {
  pub fn new(interpreter: &'a Interpreter) -> Self {
    Self {
      interpreter,
      scopes: RefCell::new(Vec::new()),
      had_error: RefCell::new(false),
      current_function: RefCell::new(FunctionType::None),
      in_while: RefCell::new(false),
    }
  }

  pub fn resolve(&self, statements: &Rc<Vec<Rc<Stmt>>>) -> Result<(), SaturdayResult> {
    for statement in statements.deref() {
      self.resolve_stmt(statement.clone())?;
    }

    Ok(())
  }

  pub fn success(&self) -> bool {
    !*self.had_error.borrow()
  }

  fn resolve_stmt(&self, stmt: Rc<Stmt>) -> Result<(), SaturdayResult> {
    stmt.accept(stmt.clone(), self)
  }

  fn resolve_expr(&self, expr: Rc<Expr>) -> Result<(), SaturdayResult> {
    expr.accept(expr.clone(), self)
  }

  fn begin_scope(&self) {
    self.scopes.borrow_mut().push(RefCell::new(HashMap::new()));
  }

  fn end_scope(&self) {
    self.scopes.borrow_mut().pop();
  }

  fn declare(&self, name: &Token) {
    if let Some(scope) = self.scopes.borrow().last() {
      if scope.borrow().contains_key(&name.as_string()) {
        self.error(name, "Already a variable with this name in this scope.");
      }

      scope.borrow_mut().insert(name.as_string(), false);
    }
  }

  fn define(&self, name: &Token) {
    if let Some(scope) = self.scopes.borrow().last() {
      scope.borrow_mut().insert(name.as_string(), true);
    }
  }

  fn resolve_local(&self, expr: Rc<Expr>, name: &Token) {
    for (scope, map) in self.scopes.borrow().iter().rev().enumerate() {
      if map.borrow().contains_key(&name.as_string()) {
        self.interpreter.resolve(expr, scope);
        return;
      }
    }
  }

  fn resolve_function(
    &self,
    function: &FunctionStmt,
    f_type: FunctionType,
  ) -> Result<(), SaturdayResult> {
    let enclosing_function = self.current_function.replace(f_type);
    self.begin_scope();

    for param in function.params.iter() {
      self.declare(param);
      self.define(param);
    }

    self.resolve(&function.body)?;
    self.end_scope();
    self.current_function.replace(enclosing_function);

    Ok(())
  }

  fn error(&self, token: &Token, message: &str) {
    self.had_error.replace(true);
    SaturdayResult::parse_error(token, message);
  }
}

impl<'a> StmtVisitor<()> for Resolver<'a> {
  fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), SaturdayResult> {
    self.begin_scope();
    self.resolve(&stmt.statements)?;
    self.end_scope();
    Ok(())
  }

  fn visit_break_stmt(&self, _: Rc<Stmt>, stmt: &BreakStmt) -> Result<(), SaturdayResult> {
    if !*self.in_while.borrow() {
      self.error(&stmt.token, "break statement outside of a while/for loop");
    }

    Ok(())
  }

  fn visit_expression_stmt(
    &self,
    _: Rc<Stmt>,
    stmt: &ExpressionStmt,
  ) -> Result<(), SaturdayResult> {
    self.resolve_expr(stmt.expression.clone())?;
    Ok(())
  }

  fn visit_function_stmt(&self, _: Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), SaturdayResult> {
    self.declare(&stmt.name);
    self.define(&stmt.name);

    self.resolve_function(stmt, FunctionType::Function)?;
    Ok(())
  }

  fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt) -> Result<(), SaturdayResult> {
    self.resolve_expr(stmt.condition.clone())?;
    self.resolve_stmt(stmt.then_branch.clone())?;
    if let Some(else_branch) = stmt.else_branch.clone() {
      self.resolve_stmt(else_branch)?;
    }

    Ok(())
  }

  fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt) -> Result<(), SaturdayResult> {
    self.resolve_expr(stmt.expression.clone())?;
    Ok(())
  }

  fn visit_return_stmt(&self, _: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), SaturdayResult> {
    if *self.current_function.borrow() == FunctionType::None {
      self.error(&stmt.keyword, "Can't return from top-level code.");
    }

    if let Some(value) = stmt.value.clone() {
      self.resolve_expr(value)?;
    }

    Ok(())
  }

  fn visit_def_stmt(&self, _: Rc<Stmt>, stmt: &DefStmt) -> Result<(), SaturdayResult> {
    self.declare(&stmt.name);
    if let Some(init) = stmt.initializer.clone() {
      self.resolve_expr(init)?;
    }

    self.define(&stmt.name);
    Ok(())
  }

  fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), SaturdayResult> {
    self.in_while.replace(true);
    self.resolve_expr(stmt.condition.clone())?;
    self.resolve_stmt(stmt.body.clone())?;
    self.in_while.replace(false);

    Ok(())
  }
}

impl<'a> ExprVisitor<()> for Resolver<'a> {
  fn visit_assign_expr(&self, wrapper: Rc<Expr>, expr: &AssignExpr) -> Result<(), SaturdayResult> {
    self.resolve_expr(expr.value.clone())?;
    self.resolve_local(wrapper, &expr.name);
    Ok(())
  }

  fn visit_binary_expr(&self, _: Rc<Expr>, expr: &BinaryExpr) -> Result<(), SaturdayResult> {
    self.resolve_expr(expr.left.clone())?;
    self.resolve_expr(expr.right.clone())?;
    Ok(())
  }

  fn visit_call_expr(&self, _: Rc<Expr>, expr: &CallExpr) -> Result<(), SaturdayResult> {
    self.resolve_expr(expr.callee.clone())?;
    for argument in expr.arguments.iter() {
      self.resolve_expr(argument.clone())?;
    }

    Ok(())
  }

  fn visit_grouping_expr(&self, _: Rc<Expr>, expr: &GroupingExpr) -> Result<(), SaturdayResult> {
    self.resolve_expr(expr.expression.clone())?;
    Ok(())
  }

  fn visit_literal_expr(&self, _: Rc<Expr>, _expr: &LiteralExpr) -> Result<(), SaturdayResult> {
    Ok(())
  }

  fn visit_logical_expr(&self, _: Rc<Expr>, expr: &LogicalExpr) -> Result<(), SaturdayResult> {
    self.resolve_expr(expr.left.clone())?;
    self.resolve_expr(expr.right.clone())?;
    Ok(())
  }

  fn visit_unary_expr(&self, _: Rc<Expr>, expr: &UnaryExpr) -> Result<(), SaturdayResult> {
    self.resolve_expr(expr.right.clone())?;
    Ok(())
  }

  fn visit_variable_expr(
    &self,
    wrapper: Rc<Expr>,
    expr: &VariableExpr,
  ) -> Result<(), SaturdayResult> {
    if !self.scopes.borrow().is_empty()
      && self
        .scopes
        .borrow()
        .last()
        .unwrap()
        .borrow()
        .get(&expr.name.as_string())
        .copied()
        == Some(false)
    {
      self.error(
        &expr.name,
        "Can't read local variable in its own initializer.",
      )
    } else {
      self.resolve_local(wrapper, &expr.name);
    }
    Ok(())
  }
}
