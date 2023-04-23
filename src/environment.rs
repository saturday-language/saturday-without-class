use crate::object::Object;
use crate::token::Token;
use crate::SaturdayResult;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
  values: HashMap<String, Object>,
  enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
  pub fn new() -> Self {
    Self {
      values: HashMap::new(),
      enclosing: None,
    }
  }

  pub fn new_with_enclosing(enclosing: Rc<RefCell<Self>>) -> Self {
    Self {
      values: HashMap::new(),
      enclosing: Some(enclosing),
    }
  }

  pub fn define(&mut self, name: &str, value: Object) {
    self.values.insert(name.to_string(), value);
  }

  pub fn get_at(&self, distance: usize, name: &str) -> Result<Object, SaturdayResult> {
    if distance == 0 {
      Ok(self.values.get(name).unwrap().clone())
    } else {
      self
        .enclosing
        .as_ref()
        .unwrap()
        .borrow()
        .get_at(distance - 1, name)
    }
  }

  pub fn get(&self, name: &Token) -> Result<Object, SaturdayResult> {
    if let Some(object) = self.values.get(&name.as_string()) {
      Ok(object.clone())
    } else if let Some(enclosing) = &self.enclosing {
      enclosing.borrow().get(name)
    } else {
      Err(SaturdayResult::runtime_error(
        name,
        &format!("Undefined variable '{}'.", name.as_string()),
      ))
    }
  }

  pub fn assign_at(
    &mut self,
    distance: usize,
    name: &Token,
    value: Object,
  ) -> Result<(), SaturdayResult> {
    if distance == 0 {
      self.values.insert(name.as_string(), value);
      Ok(())
    } else {
      self
        .enclosing
        .as_ref()
        .unwrap()
        .borrow_mut()
        .assign_at(distance - 1, name, value)
    }
  }

  pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), SaturdayResult> {
    if let Entry::Occupied(mut object) = self.values.entry(name.as_string()) {
      object.insert(value);
      Ok(())
    } else if let Some(enclosing) = &self.enclosing {
      enclosing.borrow_mut().assign(name, value)
    } else {
      Err(SaturdayResult::runtime_error(
        name,
        &format!("Undefined variable '{}'.", name.as_string()),
      ))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::token_type::TokenType;

  #[test]
  fn can_define_a_variable() {
    let mut e = Environment::new();
    e.define("One", Object::Bool(true));
    assert!(e.values.contains_key("One"));
    assert_eq!(*e.values.get("One").unwrap(), Object::Bool(true));
  }

  #[test]
  fn can_redefine_a_variable() {
    let mut e = Environment::new();
    e.define("Two", Object::Bool(true));
    e.define("Two", Object::Num(12.0));
    assert_eq!(*e.values.get("Two").unwrap(), Object::Num(12.0));
  }

  #[test]
  fn can_look_up_a_variable() {
    let mut e = Environment::new();
    e.define("Three", Object::Str("foo".to_string()));
    assert_eq!(
      e.get(&Token::new(
        TokenType::Identifier,
        "Three".to_string(),
        None,
        123,
      ))
      .ok()
      .unwrap(),
      Object::Str("foo".to_string())
    );
  }

  #[test]
  fn error_when_variable_undefined() {
    let e = Environment::new();
    let three_tok = Token::new(TokenType::Identifier, "Three".to_string(), None, 0);
    assert!(e.get(&three_tok).is_err());
  }

  #[test]
  fn error_when_assigning_to_undefined_variable() {
    let mut e = Environment::new();
    let four_tok = Token::new(TokenType::Identifier, "Four".to_string(), None, 0);
    assert!(e.assign(&four_tok, Object::Nil).is_err());
  }

  #[test]
  fn can_reassign_existing_variable() {
    let mut e = Environment::new();
    let four_tok = Token::new(TokenType::Identifier, "Four".to_string(), None, 0);
    e.define("Four", Object::Num(73.1));
    assert!(e.assign(&four_tok, Object::Num(89.5)).is_ok());
    assert_eq!(e.get(&four_tok).ok(), Some(Object::Num(89.5)));
  }

  #[test]
  fn can_enclose_an_environment() {
    let e = Rc::new(RefCell::new(Environment::new()));
    let f = Environment::new_with_enclosing(Rc::clone(&e));
    assert_eq!(f.enclosing.unwrap().borrow().values, e.borrow().values);
  }

  #[test]
  fn can_read_from_enclosed_environment() {
    let e = Rc::new(RefCell::new(Environment::new()));
    let four_tok = Token::new(TokenType::Identifier, "Four".to_string(), None, 0);
    e.borrow_mut().define("Four", Object::Num(73.1));
    let f = Environment::new_with_enclosing(Rc::clone(&e));
    assert_eq!(f.get(&four_tok).ok(), Some(Object::Num(73.1)));
  }

  #[test]
  fn can_assign_to_enclosed_environment() {
    let e = Rc::new(RefCell::new(Environment::new()));
    e.borrow_mut().define("Four", Object::Num(73.1));
    let mut f = Environment::new_with_enclosing(Rc::clone(&e));
    let four_tok = Token::new(TokenType::Identifier, "Four".to_string(), None, 0);
    assert!(f.assign(&four_tok, Object::Num(91.2)).is_ok());
    assert_eq!(f.get(&four_tok).ok(), Some(Object::Num(91.2)));
  }
}
