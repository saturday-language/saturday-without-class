use crate::callable::Callable;
use crate::environment::Environment;
use crate::error::SaturdayResult;
use crate::expr::*;
use crate::native_functions::NativeClock;
use crate::object::*;
use crate::saturday_function::SaturdayFunction;
use crate::stmt::{
  BlockStmt, BreakStmt, DefStmt, ExpressionStmt, FunctionStmt, IfStmt, PrintStmt, ReturnStmt, Stmt,
  StmtVisitor, WhileStmt,
};
use crate::token::Token;
use crate::token_type::TokenType;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Interpreter {
  pub globals: Rc<RefCell<Environment>>,
  environment: RefCell<Rc<RefCell<Environment>>>,
  locals: RefCell<HashMap<Rc<Expr>, usize>>,
}

impl StmtVisitor<()> for Interpreter {
  fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), SaturdayResult> {
    let e = Environment::new_with_enclosing(self.environment.borrow().clone());
    self.execute_block(&stmt.statements, e)
  }

  fn visit_break_stmt(&self, _: Rc<Stmt>, _: &BreakStmt) -> Result<(), SaturdayResult> {
    Err(SaturdayResult::Break)
  }

  fn visit_expression_stmt(
    &self,
    _: Rc<Stmt>,
    stmt: &ExpressionStmt,
  ) -> Result<(), SaturdayResult> {
    self.evaluate(stmt.expression.clone())?;
    Ok(())
  }

  fn visit_function_stmt(&self, _: Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), SaturdayResult> {
    let function = SaturdayFunction::new(&Rc::new(stmt), &self.environment.borrow());
    self.environment.borrow().borrow_mut().define(
      &stmt.name.as_string(),
      Object::Func(Callable {
        func: Rc::new(function),
      }),
    );
    Ok(())
  }

  fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt) -> Result<(), SaturdayResult> {
    if self.is_truthy(&self.evaluate(stmt.condition.clone())?) {
      self.execute(stmt.then_branch.clone())
    } else if let Some(else_branch) = stmt.else_branch.clone() {
      self.execute(else_branch)
    } else {
      Ok(())
    }
  }

  fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt) -> Result<(), SaturdayResult> {
    let value = self.evaluate(stmt.expression.clone())?;
    println!("{value}");
    Ok(())
  }

  fn visit_return_stmt(&self, _wrapper: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), SaturdayResult> {
    if let Some(value) = stmt.value.clone() {
      Err(SaturdayResult::return_value(self.evaluate(value)?))
    } else {
      Err(SaturdayResult::return_value(Object::Nil))
    }
  }

  fn visit_def_stmt(&self, _: Rc<Stmt>, stmt: &DefStmt) -> Result<(), SaturdayResult> {
    let value = if let Some(initializer) = stmt.initializer.clone() {
      self.evaluate(initializer)?
    } else {
      Object::Nil
    };

    self
      .environment
      .borrow()
      .borrow_mut()
      .define(&stmt.name.as_string(), value);
    Ok(())
  }

  fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), SaturdayResult> {
    while self.is_truthy(&self.evaluate(stmt.condition.clone())?) {
      match self.execute(stmt.body.clone()) {
        Err(SaturdayResult::Break) => break,
        Err(e) => return Err(e),
        Ok(_) => {}
      }
    }

    Ok(())
  }
}

impl ExprVisitor<Object> for Interpreter {
  fn visit_assign_expr(
    &self,
    wrapper: Rc<Expr>,
    expr: &AssignExpr,
  ) -> Result<Object, SaturdayResult> {
    let value = self.evaluate(expr.value.clone())?;
    if let Some(distance) = self.locals.borrow().get(&wrapper) {
      self
        .environment
        .borrow()
        .borrow_mut()
        .assign_at(*distance, &expr.name, value.clone())?;
    } else {
      self
        .globals
        .borrow_mut()
        .assign(&expr.name, value.clone())?;
    }

    Ok(value)
  }

  fn visit_binary_expr(&self, _: Rc<Expr>, expr: &BinaryExpr) -> Result<Object, SaturdayResult> {
    let left = self.evaluate(expr.left.clone())?;
    let right = self.evaluate(expr.right.clone())?;
    let op = expr.operator.token_type();

    let result = match (left, right) {
      (Object::Num(left), Object::Num(right)) => match op {
        TokenType::Minus => Object::Num(left - right),
        TokenType::Slash => Object::Num(left / right),
        TokenType::Star => Object::Num(left * right),
        TokenType::Plus => Object::Num(left + right),
        TokenType::Greater => Object::Bool(left > right),
        TokenType::GreaterEqual => Object::Bool(left >= right),
        TokenType::Less => Object::Bool(left < right),
        TokenType::LessEqual => Object::Bool(left <= right),
        TokenType::BangEqual => Object::Bool(left != right),
        TokenType::Equal => Object::Bool(left == right),
        _ => {
          todo!("need to work on your code dude")
        }
      },
      (Object::Num(left), Object::Str(right)) => match op {
        TokenType::Plus => Object::Str(format!("{left}{right}")),
        _ => Object::ArithmeticError,
      },
      (Object::Str(left), Object::Num(right)) => match op {
        TokenType::Plus => Object::Str(format!("{left}{right}")),
        _ => Object::ArithmeticError,
      },
      (Object::Str(left), Object::Str(right)) => match op {
        TokenType::Plus => Object::Str(format!("{left}{right}")),
        TokenType::BangEqual => Object::Bool(left != right),
        TokenType::Equal => Object::Bool(left == right),
        _ => Object::ArithmeticError,
      },
      (Object::Bool(left), Object::Bool(right)) => match op {
        TokenType::BangEqual => Object::Bool(left != right),
        TokenType::Equal => Object::Bool(left == right),
        _ => Object::ArithmeticError,
      },
      (Object::Nil, Object::Nil) => match op {
        TokenType::BangEqual => Object::Bool(false),
        TokenType::Equal => Object::Bool(true),
        _ => Object::ArithmeticError,
      },
      (Object::Nil, _) => match op {
        TokenType::BangEqual => Object::Bool(true),
        TokenType::Equal => Object::Bool(false),
        _ => Object::ArithmeticError,
      },
      _ => Object::ArithmeticError,
    };

    if result == Object::ArithmeticError {
      Err(SaturdayResult::runtime_error(
        &expr.operator,
        "Illegal expression",
      ))
    } else {
      Ok(result)
    }
  }

  fn visit_call_expr(&self, _: Rc<Expr>, expr: &CallExpr) -> Result<Object, SaturdayResult> {
    let callee = self.evaluate(expr.callee.clone())?;
    let mut arguments = Vec::new();
    for argument in expr.arguments.clone() {
      arguments.push(self.evaluate(argument)?);
    }

    if let Object::Func(function) = callee {
      if arguments.len() != function.func.arity() {
        return Err(SaturdayResult::runtime_error(
          &expr.paren,
          &format!(
            "Expected {} arguments but got {}.",
            function.func.arity(),
            arguments.len()
          ),
        ));
      }

      function.func.call(self, arguments)
    } else {
      Err(SaturdayResult::runtime_error(
        &expr.paren,
        "Can only call function and classes",
      ))
    }
  }

  fn visit_grouping_expr(
    &self,
    _: Rc<Expr>,
    expr: &GroupingExpr,
  ) -> Result<Object, SaturdayResult> {
    self.evaluate(expr.expression.clone())
  }

  fn visit_literal_expr(&self, _: Rc<Expr>, expr: &LiteralExpr) -> Result<Object, SaturdayResult> {
    Ok(expr.value.clone().unwrap())
  }

  fn visit_logical_expr(&self, _: Rc<Expr>, expr: &LogicalExpr) -> Result<Object, SaturdayResult> {
    let left = self.evaluate(expr.left.clone())?;

    if expr.operator.is(TokenType::Or) {
      if self.is_truthy(&left) {
        return Ok(left);
      }
    } else if !self.is_truthy(&left) {
      return Ok(left);
    }

    self.evaluate(expr.right.clone())
  }

  fn visit_unary_expr(&self, _: Rc<Expr>, expr: &UnaryExpr) -> Result<Object, SaturdayResult> {
    let right = self.evaluate(expr.right.clone())?;
    match expr.operator.token_type() {
      TokenType::Minus => match right {
        Object::Num(n) => Ok(Object::Num(-n)),
        _ => Ok(Object::Nil),
      },
      TokenType::Bang => Ok(Object::Bool(!self.is_truthy(&right))),
      _ => Err(SaturdayResult::error(
        expr.operator.line,
        "Unreachable according to Nystrom",
      )),
    }
  }

  fn visit_variable_expr(
    &self,
    wrapper: Rc<Expr>,
    expr: &VariableExpr,
  ) -> Result<Object, SaturdayResult> {
    // self.environment.borrow().borrow().get(&expr.name)
    self.look_up_variable(&expr.name, wrapper)
  }
}

impl Interpreter {
  pub fn new() -> Self {
    let globals = Rc::new(RefCell::new(Environment::new()));
    globals.borrow_mut().define(
      "clock",
      Object::Func(Callable {
        func: Rc::new(NativeClock {}),
      }),
    );

    Self {
      globals: Rc::clone(&globals),
      environment: RefCell::new(Rc::clone(&globals)),
      locals: RefCell::new(HashMap::new()),
    }
  }

  fn evaluate(&self, expr: Rc<Expr>) -> Result<Object, SaturdayResult> {
    expr.accept(expr.clone(), self)
  }

  fn execute(&self, stmt: Rc<Stmt>) -> Result<(), SaturdayResult> {
    stmt.accept(stmt.clone(), self)
  }

  pub fn execute_block(
    &self,
    statements: &Rc<Vec<Rc<Stmt>>>,
    environment: Environment,
  ) -> Result<(), SaturdayResult> {
    let previous = self.environment.replace(Rc::new(RefCell::new(environment)));
    let result = statements
      .iter()
      .try_for_each(|statement| self.execute(statement.clone()));
    self.environment.replace(previous);
    result
  }

  /// 任何不等于Nil和False的识别为true
  fn is_truthy(&self, object: &Object) -> bool {
    !matches!(object, Object::Nil | Object::Bool(false))
  }

  pub fn interpreter(&self, statements: &[Rc<Stmt>]) -> bool {
    let mut success = true;
    for statement in statements {
      if self.execute(statement.clone()).is_err() {
        success = false;
        break;
      }
    }

    success
  }

  pub fn print_environment(&self) {
    println!("{:?}", self.environment.borrow().borrow());
  }

  pub fn resolve(&self, expr: Rc<Expr>, depth: usize) {
    self.locals.borrow_mut().insert(expr, depth);
  }

  pub fn look_up_variable(&self, name: &Token, expr: Rc<Expr>) -> Result<Object, SaturdayResult> {
    if let Some(distance) = self.locals.borrow().get(&expr) {
      self
        .environment
        .borrow()
        .borrow()
        .get_at(*distance, &name.as_string())
    } else {
      self.globals.borrow().get(name)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::token::Token;

  fn make_literal(o: Object) -> Box<Expr> {
    Box::new(Expr::Literal(LiteralExpr { value: Some(o) }))
  }

  fn make_literal_string(s: &str) -> Box<Expr> {
    make_literal(Object::Str(s.to_string()))
  }

  #[test]
  fn test_unary_minus() {
    let terp = Interpreter::new();
    let unary_expr = UnaryExpr {
      operator: Token::new(TokenType::Minus, "-".to_string(), None, 123),
      right: make_literal(Object::Num(123.0)),
    };
    let result = terp.visit_unary_expr(&unary_expr);
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(Object::Num(-123.0)));
  }

  #[test]
  fn test_unary_not() {
    let terp = Interpreter::new();
    let unary_expr = UnaryExpr {
      operator: Token::new(TokenType::Bang, "!".to_string(), None, 123),
      right: make_literal(Object::Bool(false)),
    };
    let result = terp.visit_unary_expr(&unary_expr);
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(Object::Bool(true)));
  }

  #[test]
  fn test_subtraction() {
    let terp = Interpreter::new();
    let binary_expr = BinaryExpr {
      left: make_literal(Object::Num(15.0)),
      operator: Token::new(TokenType::Minus, "-".to_string(), None, 123),
      right: make_literal(Object::Num(7.0)),
    };
    let result = terp.visit_binary_expr(&binary_expr);
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(Object::Num(8.0)));
  }

  #[test]
  fn test_division() {
    let terp = Interpreter::new();
    let binary_expr = BinaryExpr {
      left: make_literal(Object::Num(21.0)),
      operator: Token::new(TokenType::Slash, "/".to_string(), None, 123),
      right: make_literal(Object::Num(7.0)),
    };
    let result = terp.visit_binary_expr(&binary_expr);
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(Object::Num(3.0)));
  }

  #[test]
  fn test_multiplication() {
    let terp = Interpreter::new();
    let binary_expr = BinaryExpr {
      left: make_literal(Object::Num(15.0)),
      operator: Token::new(TokenType::Star, "*".to_string(), None, 123),
      right: make_literal(Object::Num(7.0)),
    };
    let result = terp.visit_binary_expr(&binary_expr);
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(Object::Num(105.0)));
  }

  #[test]
  fn test_addition() {
    let terp = Interpreter::new();
    let binary_expr = BinaryExpr {
      left: make_literal(Object::Num(15.0)),
      operator: Token::new(TokenType::Plus, "+".to_string(), None, 123),
      right: make_literal(Object::Num(7.0)),
    };
    let result = terp.visit_binary_expr(&binary_expr);
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(Object::Num(22.0)));
  }

  #[test]
  fn test_string_concatenation() {
    let terp = Interpreter::new();
    let binary_expr = BinaryExpr {
      left: make_literal_string("hello, "),
      operator: Token::new(TokenType::Plus, "+".to_string(), None, 123),
      right: make_literal_string("world!"),
    };
    let result = terp.visit_binary_expr(&binary_expr);
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(Object::Str("hello, world!".to_string())));
  }

  #[test]
  fn test_arithmetic_error_for_subtraction() {
    let terp = Interpreter::new();
    let binary_expr = BinaryExpr {
      left: make_literal(Object::Num(15.0)),
      operator: Token::new(TokenType::Minus, "-".to_string(), None, 123),
      right: make_literal(Object::Bool(true)),
    };
    let result = terp.visit_binary_expr(&binary_expr);
    assert!(result.is_err());
  }

  #[test]
  fn test_arithmetic_error_for_greater() {
    let terp = Interpreter::new();
    let binary_expr = BinaryExpr {
      left: make_literal(Object::Num(15.0)),
      operator: Token::new(TokenType::Greater, ">".to_string(), None, 123),
      right: make_literal(Object::Bool(true)),
    };
    let result = terp.visit_binary_expr(&binary_expr);
    assert!(result.is_err());
  }

  #[test]
  fn test_equals() {
    run_comparison_test(
      Token::new(TokenType::Equal, "==".to_string(), None, 123),
      vec![false, true, false],
    );
  }

  #[test]
  fn test_not_equals() {
    run_comparison_test(
      Token::new(TokenType::BangEqual, "!=".to_string(), None, 123),
      vec![true, false, true],
    );
  }

  #[test]
  fn test_not_equals_string() {
    let terp = Interpreter::new();
    let binary_expr = BinaryExpr {
      left: make_literal_string("hello"),
      operator: Token::new(TokenType::Equal, "==".to_string(), None, 123),
      right: make_literal_string("hellx"),
    };
    let result = terp.visit_binary_expr(&binary_expr);
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(Object::Bool(false)));
  }

  #[test]
  fn test_equals_string() {
    let terp = Interpreter::new();
    let binary_expr = BinaryExpr {
      left: make_literal_string("world"),
      operator: Token::new(TokenType::Equal, "==".to_string(), None, 123),
      right: make_literal_string("world"),
    };
    let result = terp.visit_binary_expr(&binary_expr);
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(Object::Bool(true)));
  }

  #[test]
  fn test_equals_nil() {
    let terp = Interpreter::new();
    let binary_expr = BinaryExpr {
      left: make_literal(Object::Nil),
      operator: Token::new(TokenType::Equal, "==".to_string(), None, 123),
      right: make_literal(Object::Nil),
    };
    let result = terp.visit_binary_expr(&binary_expr);
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(Object::Bool(true)));
  }

  fn run_comparison_test(tok: Token, cmps: Vec<bool>) {
    let nums = vec![14.0, 15.0, 16.0];
    let terp = Interpreter::new();

    for (c, nums) in cmps.iter().zip(nums) {
      let binary_expr = BinaryExpr {
        left: make_literal(Object::Num(nums)),
        operator: tok.dup(),
        right: make_literal(Object::Num(15.0)),
      };
      let result = terp.visit_binary_expr(&binary_expr);
      assert!(result.is_ok());
      assert_eq!(
        result.ok(),
        Some(Object::Bool(*c)),
        "Testing {} {} 15.0",
        nums,
        tok.lexeme
      );
    }
  }

  #[test]
  fn test_less_than() {
    run_comparison_test(
      Token::new(TokenType::Less, "<".to_string(), None, 123),
      vec![true, false, false],
    );
  }

  #[test]
  fn test_less_or_equal_to() {
    run_comparison_test(
      Token::new(TokenType::LessEqual, "<=".to_string(), None, 123),
      vec![true, true, false],
    );
  }

  #[test]
  fn test_greater_than() {
    run_comparison_test(
      Token::new(TokenType::Greater, ">".to_string(), None, 123),
      vec![false, false, true],
    );
  }

  #[test]
  fn test_greater_or_equal_to() {
    run_comparison_test(
      Token::new(TokenType::GreaterEqual, ">=".to_string(), None, 123),
      vec![false, true, true],
    );
  }

  #[test]
  fn test_var_stmt_with_initializer() {
    let terp = Interpreter::new();
    let name = Token::new(TokenType::Identifier, "foo".to_string(), None, 123);
    let def_stmt = DefStmt {
      name: name.dup(),
      initializer: Some(*make_literal(Object::Num(23.0))),
    };
    assert!(terp.visit_def_stmt(&def_stmt).is_ok());
    assert_eq!(
      terp.environment.borrow().borrow().get(&name).ok(),
      Some(Object::Num(23.0))
    );
  }

  #[test]
  fn test_var_stmt_without_initializer() {
    let terp = Interpreter::new();
    let name = Token::new(TokenType::Identifier, "foo".to_string(), None, 123);
    let def_stmt = DefStmt {
      name: name.dup(),
      initializer: None,
    };
    assert!(terp.visit_def_stmt(&def_stmt).is_ok());
    assert_eq!(
      terp.environment.borrow().borrow().get(&name).ok(),
      Some(Object::Nil)
    );
  }

  #[test]
  fn test_variable_expr() {
    let terp = Interpreter::new();
    let name = Token::new(TokenType::Identifier, "foo".to_string(), None, 123);
    let def_stmt = DefStmt {
      name: name.dup(),
      initializer: Some(*make_literal(Object::Num(23.0))),
    };

    assert!(terp.visit_def_stmt(&def_stmt).is_ok());

    let def_expr = VariableExpr { name: name.dup() };
    assert_eq!(
      terp.visit_variable_expr(&def_expr).ok(),
      Some(Object::Num(23.0))
    );
  }

  #[test]
  fn test_undefined_variable_expr() {
    let terp = Interpreter::new();
    let name = Token::new(TokenType::Identifier, "foo".to_string(), None, 123);
    let def_expr = VariableExpr { name: name.dup() };
    assert!(terp.visit_variable_expr(&def_expr).is_err());
  }
}
