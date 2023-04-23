use crate::callable::SaturdayCallable;
use crate::environment::Environment;
use crate::error::SaturdayResult;
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::stmt::{FunctionStmt, Stmt};
use crate::token::Token;
use std::cell::RefCell;
use std::rc::Rc;

pub struct SaturdayFunction {
  name: Token,
  params: Rc<Vec<Token>>,
  body: Rc<Vec<Rc<Stmt>>>,
  closure: Rc<RefCell<Environment>>,
}

impl SaturdayFunction {
  pub fn new(declaration: &FunctionStmt, closure: &Rc<RefCell<Environment>>) -> Self {
    Self {
      name: declaration.name.dup(),
      params: Rc::clone(&declaration.params),
      body: Rc::clone(&declaration.body),
      closure: Rc::clone(closure),
    }
  }
}

impl SaturdayCallable for SaturdayFunction {
  fn call(
    &self,
    interpreter: &Interpreter,
    arguments: Vec<Object>,
  ) -> Result<Object, SaturdayResult> {
    let mut e = Environment::new_with_enclosing(Rc::clone(&self.closure));
    for (param, arg) in self.params.iter().zip(arguments.iter()) {
      e.define(&param.as_string(), arg.clone());
    }

    match interpreter.execute_block(&self.body, e) {
      Err(SaturdayResult::ReturnValue { value }) => Ok(value),
      Err(e) => Err(e),
      Ok(_) => Ok(Object::Nil),
    }
  }

  fn arity(&self) -> usize {
    self.params.len()
  }

  fn to_string(&self) -> String {
    self.name.as_string()
  }
}
