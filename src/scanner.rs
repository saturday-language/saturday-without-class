use crate::object::Object;
use crate::token::Token;
use crate::SaturdayResult;

use super::token_type::*;

pub struct Scanner {
  source: Vec<char>,
  tokens: Vec<Token>,
  start: usize,
  current: usize,
  line: usize,
}

impl Scanner {
  pub fn new(source: String) -> Scanner {
    Scanner {
      source: source.chars().collect(),
      tokens: Vec::new(),
      start: 0,
      current: 0,
      line: 1,
    }
  }

  /// # 开始解析token
  /// ```
  /// 通过scan_token逐个解析
  /// ```
  pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, SaturdayResult> {
    let mut had_error: Option<SaturdayResult> = None;
    while !self.is_at_end() {
      self.start = self.current;
      match self.scan_token() {
        Ok(()) => {}
        Err(e) => {
          had_error = Some(e);
        }
      }
    }

    self.tokens.push(Token::eof(self.line));
    if let Some(e) = had_error {
      Err(e)
    } else {
      Ok(&self.tokens)
    }
  }

  /// 判断内容是否解析完成
  fn is_at_end(&self) -> bool {
    self.current >= self.source.len()
  }

  /// 扫描单个token
  fn scan_token(&mut self) -> Result<(), SaturdayResult> {
    let c = self.advance();
    match c {
      '(' => self.add_token(TokenType::LeftParen),
      ')' => self.add_token(TokenType::RightParen),
      '{' => self.add_token(TokenType::LeftBrace),
      '}' => self.add_token(TokenType::RightBrace),
      ',' => self.add_token(TokenType::Comma),
      '.' => self.add_token(TokenType::Dot),
      '-' => self.add_token(TokenType::Minus),
      '+' => self.add_token(TokenType::Plus),
      ';' => self.add_token(TokenType::SemiColon),
      '*' => self.add_token(TokenType::Star),
      '!' => {
        let tok = if self.r#match('=') {
          TokenType::BangEqual
        } else {
          TokenType::Bang
        };

        self.add_token(tok);
      }
      '=' => {
        let tok = if self.r#match('=') {
          TokenType::Equal
        } else {
          TokenType::Assign
        };

        self.add_token(tok);
      }
      '<' => {
        let tok = if self.r#match('=') {
          TokenType::LessEqual
        } else {
          TokenType::Less
        };

        self.add_token(tok);
      }
      '>' => {
        let tok = if self.r#match('=') {
          TokenType::GreaterEqual
        } else {
          TokenType::Greater
        };

        self.add_token(tok);
      }
      '/' => {
        if self.r#match('/') {
          // 匹配单行注释
          while let Some(ch) = self.peek() {
            if ch != '\n' {
              self.advance();
            } else {
              break;
            }
          }
        } else if self.r#match('*') {
          // 匹配块级注释
          self.scan_comment()?;
        } else {
          self.add_token(TokenType::Slash);
        }
      }
      ' ' | '\r' | '\t' => {}
      '\n' => {
        self.line += 1;
      }
      '"' => {
        self.string()?;
      }
      '0'..='9' => {
        self.number();
      }
      _ if c.is_alphabetic() || c == '_' => {
        self.identifier();
      }
      _ => {
        return Err(SaturdayResult::error(self.line, "Unexpected character"));
      }
    }

    Ok(())
  }

  fn scan_comment(&mut self) -> Result<(), SaturdayResult> {
    loop {
      match self.peek() {
        Some('*') => {
          self.advance();
          if self.r#match('/') {
            return Ok(());
          }
        }
        Some('/') => {
          self.advance();
          if self.r#match('*') {
            self.scan_comment()?;
          }
        }
        Some('\n') => {
          self.advance();
          self.line += 1;
        }
        None => {
          return Err(SaturdayResult::error(self.line, "Unterminated comment"));
        }
        _ => {
          self.advance();
        }
      }
    }
  }

  fn is_alpha_numeric(ch: Option<char>) -> bool {
    if let Some(ch) = ch {
      ch.is_ascii_alphanumeric() || ch == '_'
    } else {
      false
    }
  }

  fn identifier(&mut self) {
    while Scanner::is_alpha_numeric(self.peek()) {
      self.advance();
    }

    let text: String = self.source[self.start..self.current].iter().collect();
    if let Some(t_type) = Scanner::keywords(&text) {
      self.add_token(t_type);
    } else {
      self.add_token(TokenType::Identifier);
    }
  }

  fn number(&mut self) {
    while Scanner::is_digit(self.peek()) {
      self.advance();
    }

    if self.peek() == Some('.') && Scanner::is_digit(self.peek_next()) {
      // consume the "."
      self.advance();

      while Scanner::is_digit(self.peek()) {
        self.advance();
      }
    }

    let value: String = self.source[self.start..self.current].iter().collect();
    let value: f64 = value.parse().unwrap();
    self.add_token_object(TokenType::Number, Some(Object::Num(value)));
  }

  fn peek_next(&self) -> Option<char> {
    self.source.get(self.current + 1).copied()
  }

  fn is_digit(ch: Option<char>) -> bool {
    if let Some(ch) = ch {
      ch.is_ascii_digit()
    } else {
      false
    }
  }

  fn string(&mut self) -> Result<(), SaturdayResult> {
    while let Some(ch) = self.peek() {
      match ch {
        '"' => {
          break;
        }
        '\n' => {
          self.line += 1;
        }
        _ => {}
      }
      self.advance();
    }

    if self.is_at_end() {
      return Err(SaturdayResult::error(self.line, "Unterminated string."));
    }

    self.advance();
    // TODO: handle escape sequence
    let value: String = self.source[self.start + 1..self.current - 1]
      .iter()
      .collect();
    self.add_token_object(TokenType::String, Some(Object::Str(value)));
    Ok(())
  }

  fn advance(&mut self) -> char {
    let result = *self.source.get(self.current).unwrap();
    self.current += 1;
    result
  }

  fn add_token(&mut self, t_type: TokenType) {
    self.add_token_object(t_type, None);
  }

  fn add_token_object(&mut self, t_type: TokenType, literal: Option<Object>) {
    let lexeme = self.source[self.start..self.current].iter().collect();
    self
      .tokens
      .push(Token::new(t_type, lexeme, literal, self.line));
  }

  fn r#match(&mut self, expected: char) -> bool {
    match self.source.get(self.current) {
      Some(ch) if *ch == expected => {
        self.advance();
        true
      }
      _ => false,
    }
  }

  fn peek(&self) -> Option<char> {
    self.source.get(self.current).copied()
  }

  fn keywords(check: &str) -> Option<TokenType> {
    match check {
      "and" => Some(TokenType::And),
      "class" => Some(TokenType::Class),
      "else" => Some(TokenType::Else),
      "false" => Some(TokenType::False),
      "for" => Some(TokenType::For),
      "fun" => Some(TokenType::Fun),
      "if" => Some(TokenType::If),
      "nil" => Some(TokenType::Nil),
      "or" => Some(TokenType::Or),
      "print" => Some(TokenType::Print),
      "return" => Some(TokenType::Return),
      "super" => Some(TokenType::Super),
      "this" => Some(TokenType::This),
      "true" => Some(TokenType::True),
      "var" => Some(TokenType::Var),
      "while" => Some(TokenType::While),
      "def" => Some(TokenType::Def),
      "break" => Some(TokenType::Break),
      _ => None,
    }
  }
}
