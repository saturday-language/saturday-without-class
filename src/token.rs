use super::token_type::*;
use crate::object::Object;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct Token {
  pub t_type: TokenType,
  pub lexeme: String,
  pub literal: Option<Object>,
  pub line: usize,
}

impl Token {
  pub fn new(t_type: TokenType, lexeme: String, literal: Option<Object>, line: usize) -> Token {
    Token {
      t_type,
      lexeme,
      literal,
      line,
    }
  }

  pub fn is(&self, t_type: TokenType) -> bool {
    self.t_type == t_type
  }

  pub fn token_type(&self) -> TokenType {
    self.t_type
  }

  pub fn as_string(&self) -> String {
    self.lexeme.clone()
  }

  pub fn dup(&self) -> Self {
    Token {
      t_type: self.t_type,
      lexeme: self.lexeme.to_string(),
      literal: self.literal.clone(),
      line: self.line,
    }
  }

  /// # 空令牌
  /// （空Token） 表示解析结束
  pub fn eof(line: usize) -> Token {
    Token {
      t_type: TokenType::Eof,
      lexeme: "".to_string(),
      literal: None,
      line,
    }
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{:?} {} {}",
      self.t_type,
      self.lexeme,
      if let Some(literal) = &self.literal {
        literal.to_string()
      } else {
        "None".to_string()
      }
    )
  }
}
