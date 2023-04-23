use crate::error::*;
use crate::expr::*;

pub struct AstPrinter;

impl AstPrinter {
  pub fn print(&self, expr: &Expr) -> Result<String, SaturdayError> {
    expr.accept(self)
  }

  fn parenthesize(&self, name: &String, expr_list: &[&Box<Expr>]) -> Result<String, SaturdayError> {
    let mut builder = format!("({name}");

    for expr in expr_list {
      builder = format!("{builder} {}", expr.accept(self)?);
    }

    builder = format!("{builder})");
    Ok(builder)
  }
}

impl ExprVisitor<String> for AstPrinter {
  fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<String, SaturdayError> {
    self.parenthesize(&expr.operator.lexeme, &[&expr.left, &expr.right])
  }

  fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<String, SaturdayError> {
    self.parenthesize(&"group".to_string(), &[&expr.expression])
  }

  fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<String, SaturdayError> {
    if let Some(value) = &expr.value {
      Ok(value.to_string())
    } else {
      Ok("nil".to_string())
    }
  }

  fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<String, SaturdayError> {
    self.parenthesize(&expr.operator.lexeme, &[&expr.right])
  }

  fn visit_variable_expr(&self, expr: &VariableExpr) -> Result<String, SaturdayError> {
    todo!()
  }
}
