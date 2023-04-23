use crate::callable::SaturdayCallable;
use crate::error::SaturdayResult;
use crate::interpreter::Interpreter;
use crate::object::Object;
use std::time::SystemTime;

pub struct NativeClock;

impl SaturdayCallable for NativeClock {
  fn call(
    &self,
    _interpreter: &Interpreter,
    _arguments: Vec<Object>,
  ) -> Result<Object, SaturdayResult> {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
      Ok(n) => Ok(Object::Num(n.as_millis() as f64)),
      Err(e) => Err(SaturdayResult::system_error(&format!(
        "Clock returned invalid duration: {:?}",
        e
      ))),
    }
  }

  fn arity(&self) -> usize {
    0
  }

  fn to_string(&self) -> String {
    String::from("Native:Clock")
  }
}
