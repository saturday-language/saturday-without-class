#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
  LeftParen,  // (
  RightParen, // )
  LeftBrace,  // {
  RightBrace, // }
  Comma,
  Dot,
  Minus,
  Plus,
  SemiColon,
  Slash,
  Star,
  Bang,      // !
  BangEqual, // !=
  Assign,    // Assign ('=')
  Equal,     // Equal ('==')
  Greater,
  GreaterEqual,
  Less,
  LessEqual,
  Identifier,
  String,
  Number,
  And,
  Class,
  Else,
  False,
  Fun,
  For,
  If,
  Nil,
  Or,
  Print,
  Return,
  Super,
  This,
  True,
  Var,
  Def,
  While,
  Eof,
  Break,
}
