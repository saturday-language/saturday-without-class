use crate::object::Object;
use crate::token::Token;
use crate::token_type::TokenType;

pub enum SaturdayResult {
  ParseError { token: Token, message: String },
  RuntimeError { token: Token, message: String },
  Error { line: usize, message: String },
  SystemError { message: String },
  ReturnValue { value: Object },
  Break,
  Fail,
}

impl SaturdayResult {
  pub fn fail() -> SaturdayResult {
    SaturdayResult::Fail
  }

  pub fn return_value(value: Object) -> SaturdayResult {
    SaturdayResult::ReturnValue { value }
  }

  pub fn error(line: usize, message: &str) -> Self {
    let err = Self::Error {
      line,
      message: message.to_string(),
    };
    err.report("");
    err
  }

  pub fn runtime_error(token: &Token, message: &str) -> Self {
    let err = Self::RuntimeError {
      token: token.dup(),
      message: message.to_string(),
    };
    err.report("");
    err
  }

  pub fn parse_error(token: &Token, message: &str) -> Self {
    let err = Self::ParseError {
      token: token.dup(),
      message: message.to_string(),
    };
    err.report("");
    err
  }

  pub fn system_error(message: &str) -> Self {
    let err = SaturdayResult::SystemError {
      message: message.to_string(),
    };
    err.report("");
    err
  }

  fn report(&self, loc: &str) {
    match self {
      Self::ParseError { token, message } => {
        eprintln!(
          "[line {}] Error at '{}': {}",
          token.line,
          token.as_string(),
          message
        );
      }
      Self::RuntimeError { token, message } => {
        if token.is(TokenType::Eof) {
          eprintln!("[line {}] Error at end: {}", token.line, message);
        } else {
          eprintln!("{}\n[line {}]", message, token.line);
        }
      }
      Self::SystemError { message } => {
        eprintln!("System Error: {message}");
      }
      Self::Error { line, message } => {
        eprintln!("[line {}] Error{}: {}", line, loc, message);
      }
      Self::Break | Self::ReturnValue { .. } => {}
      Self::Fail => {
        panic!("should not get here")
      }
    };
  }
}
