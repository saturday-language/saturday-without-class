use crate::object::Object;
use crate::Interpreter;
use crate::SaturdayResult;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use std::vec::Vec;

#[derive(Clone)]
pub struct Callable {
  pub func: Rc<dyn SaturdayCallable>,
}

impl Debug for Callable {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", SaturdayCallable::to_string(self))
  }
}

impl Display for Callable {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", SaturdayCallable::to_string(self))
  }
}

impl PartialEq for Callable {
  fn eq(&self, other: &Self) -> bool {
    std::ptr::eq(
      Rc::as_ptr(&self.func) as *const (),
      Rc::as_ptr(&other.func) as *const (),
    )
  }
}

pub trait SaturdayCallable {
  fn call(
    &self,
    interpreter: &Interpreter,
    arguments: Vec<Object>,
  ) -> Result<Object, SaturdayResult>;
  fn arity(&self) -> usize;
  fn to_string(&self) -> String;
}

impl SaturdayCallable for Callable {
  fn call(
    &self,
    interpreter: &Interpreter,
    arguments: Vec<Object>,
  ) -> Result<Object, SaturdayResult> {
    self.func.call(interpreter, arguments)
  }

  fn arity(&self) -> usize {
    self.func.arity()
  }

  fn to_string(&self) -> String {
    self.func.to_string()
  }
}
