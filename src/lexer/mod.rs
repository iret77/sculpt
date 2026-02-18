use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
  Identifier(String),
  Number(f64),
  String(String),
  Keyword(Keyword),
  Gt,
  PlusEq,
  Eq,
  Gte,
  LParen,
  RParen,
  Comma,
  Colon,
  Newline,
  Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
  Module,
  Flow,
  State,
  Rule,
  Nd,
  Propose,
  Satisfy,
  On,
  When,
  Emit,
  Run,
  Terminate,
  Start,
  End,
  Null,
}

#[derive(Debug, Clone)]
pub struct Token {
  pub kind: TokenKind,
  pub line: usize,
  pub col: usize,
}

pub fn lex(input: &str) -> Result<Vec<Token>> {
  let mut tokens = Vec::new();
  let mut chars = input.chars().peekable();
  let mut line = 1usize;
  let mut col = 1usize;

  while let Some(ch) = chars.peek().cloned() {
    match ch {
      ' ' | '\t' | '\r' => {
        chars.next();
        col += 1;
      }
      '\n' => {
        chars.next();
        tokens.push(Token { kind: TokenKind::Newline, line, col });
        line += 1;
        col = 1;
      }
      '#' => {
        while let Some(c) = chars.next() {
          if c == '\n' { 
            tokens.push(Token { kind: TokenKind::Newline, line, col });
            line += 1;
            col = 1;
            break;
          }
        }
      }
      '+' => {
        chars.next();
        if chars.peek() == Some(&'=') {
          chars.next();
          tokens.push(Token { kind: TokenKind::PlusEq, line, col });
          col += 2;
        } else {
          bail!("Unexpected '+' at {}:{}", line, col);
        }
      }
      '=' => {
        chars.next();
        if chars.peek() == Some(&'=') {
          bail!("Unsupported '==' at {}:{}", line, col);
        } else {
          tokens.push(Token { kind: TokenKind::Eq, line, col });
          col += 1;
        }
      }
      '>' => {
        chars.next();
        if chars.peek() == Some(&'=') {
          chars.next();
          tokens.push(Token { kind: TokenKind::Gte, line, col });
          col += 2;
        } else {
          tokens.push(Token { kind: TokenKind::Gt, line, col });
          col += 1;
        }
      }
      '(' => {
        chars.next();
        tokens.push(Token { kind: TokenKind::LParen, line, col });
        col += 1;
      }
      ')' => {
        chars.next();
        tokens.push(Token { kind: TokenKind::RParen, line, col });
        col += 1;
      }
      ',' => {
        chars.next();
        tokens.push(Token { kind: TokenKind::Comma, line, col });
        col += 1;
      }
      ':' => {
        chars.next();
        tokens.push(Token { kind: TokenKind::Colon, line, col });
        col += 1;
      }
      '"' => {
        chars.next();
        col += 1;
        let mut s = String::new();
        while let Some(c) = chars.next() {
          col += 1;
          match c {
            '"' => break,
            '\\' => {
              if let Some(esc) = chars.next() {
                col += 1;
                match esc {
                  'n' => s.push('\n'),
                  't' => s.push('\t'),
                  '"' => s.push('"'),
                  '\\' => s.push('\\'),
                  _ => s.push(esc),
                }
              }
            }
            _ => s.push(c),
          }
        }
        tokens.push(Token { kind: TokenKind::String(s), line, col });
      }
      c if c.is_ascii_digit() => {
        let start_col = col;
        let mut num = String::new();
        while let Some(c2) = chars.peek().cloned() {
          if c2.is_ascii_digit() || c2 == '.' {
            num.push(c2);
            chars.next();
            col += 1;
          } else {
            break;
          }
        }
        let value: f64 = num.parse()?;
        tokens.push(Token { kind: TokenKind::Number(value), line, col: start_col });
      }
      c if is_ident_start(c) => {
        let start_col = col;
        let mut ident = String::new();
        while let Some(c2) = chars.peek().cloned() {
          if is_ident_continue(c2) {
            ident.push(c2);
            chars.next();
            col += 1;
          } else {
            break;
          }
        }
        let kind = match ident.as_str() {
          "module" => TokenKind::Keyword(Keyword::Module),
          "flow" => TokenKind::Keyword(Keyword::Flow),
          "state" => TokenKind::Keyword(Keyword::State),
          "rule" => TokenKind::Keyword(Keyword::Rule),
          "nd" => TokenKind::Keyword(Keyword::Nd),
          "propose" => TokenKind::Keyword(Keyword::Propose),
          "satisfy" => TokenKind::Keyword(Keyword::Satisfy),
          "on" => TokenKind::Keyword(Keyword::On),
          "when" => TokenKind::Keyword(Keyword::When),
          "emit" => TokenKind::Keyword(Keyword::Emit),
          "run" => TokenKind::Keyword(Keyword::Run),
          "terminate" => TokenKind::Keyword(Keyword::Terminate),
          "start" => TokenKind::Keyword(Keyword::Start),
          "end" => TokenKind::Keyword(Keyword::End),
          "null" => TokenKind::Keyword(Keyword::Null),
          _ => TokenKind::Identifier(ident),
        };
        tokens.push(Token { kind, line, col: start_col });
      }
      _ => {
        bail!("Unexpected character '{}' at {}:{}", ch, line, col);
      }
    }
  }

  tokens.push(Token { kind: TokenKind::Eof, line, col });
  Ok(tokens)
}

fn is_ident_start(c: char) -> bool {
  c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
  c.is_ascii_alphanumeric() || c == '_'
}
