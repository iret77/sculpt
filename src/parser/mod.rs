use anyhow::{bail, Result};

use crate::ast::*;
use crate::lexer::{lex, Keyword, Token, TokenKind};

pub fn parse_source(input: &str) -> Result<Module> {
  let tokens = lex(input)?;
  let mut parser = Parser { tokens, pos: 0 };
  parser.parse_module()
}

struct Parser {
  tokens: Vec<Token>,
  pos: usize,
}

impl Parser {
  fn parse_module(&mut self) -> Result<Module> {
    self.consume_newlines();
    self.expect_keyword(Keyword::Module)?;
    self.expect(TokenKind::LParen)?;
    let name = self.expect_ident()?;
    self.expect(TokenKind::RParen)?;
    let mut items = Vec::new();
    self.consume_newlines();

    while !self.check_keyword(Keyword::End) && !self.is_eof() {
      if self.check_keyword(Keyword::Flow) {
        items.push(Item::Flow(self.parse_flow()?));
      } else if self.check_keyword(Keyword::State) {
        items.push(Item::GlobalState(self.parse_global_state()?));
      } else if self.check_keyword(Keyword::Rule) {
        items.push(Item::Rule(self.parse_rule()?));
      } else if self.check_keyword(Keyword::Nd) {
        items.push(Item::Nd(self.parse_nd()?));
      } else if self.check(TokenKind::Newline) {
        self.consume_newlines();
      } else {
        let tok = self.peek().cloned();
        bail!("Unexpected token {:?} at module level", tok.map(|t| t.kind));
      }
      self.consume_newlines();
    }

    self.expect_keyword(Keyword::End)?;
    Ok(Module { name, items })
  }

  fn parse_flow(&mut self) -> Result<Flow> {
    self.expect_keyword(Keyword::Flow)?;
    self.expect(TokenKind::LParen)?;
    let name = self.expect_ident()?;
    self.expect(TokenKind::RParen)?;
    let mut start = None;
    let mut states = Vec::new();
    self.consume_newlines();

    while !self.check_keyword(Keyword::End) && !self.is_eof() {
      if self.check_keyword(Keyword::Start) {
        self.expect_keyword(Keyword::Start)?;
        self.expect_transition()?;
        let target = self.expect_ident()?;
        start = Some(target);
      } else if self.check_keyword(Keyword::State) {
        states.push(self.parse_state_block(true)?);
      } else if self.check(TokenKind::Newline) {
        self.consume_newlines();
      } else {
        let tok = self.peek().cloned();
        bail!("Unexpected token {:?} in flow", tok.map(|t| t.kind));
      }
      self.consume_newlines();
    }

    self.expect_keyword(Keyword::End)?;
    Ok(Flow { name, start, states })
  }

  fn parse_global_state(&mut self) -> Result<StateBlock> {
    self.expect_keyword(Keyword::State)?;
    if self.check(TokenKind::LParen) {
      self.advance();
      if !self.check(TokenKind::RParen) {
        bail!("Global state blocks must be state() with no name");
      }
      self.expect(TokenKind::RParen)?;
    } else {
      bail!("Global state blocks must be state()");
    }
    self.consume_newlines();
    let statements = self.parse_state_statements(false)?;
    self.expect_keyword(Keyword::End)?;
    Ok(StateBlock { name: None, statements })
  }

  fn parse_state_block(&mut self, requires_name: bool) -> Result<StateBlock> {
    self.expect_keyword(Keyword::State)?;
    let name = if self.check(TokenKind::LParen) {
      self.advance();
      if self.check(TokenKind::RParen) {
        self.advance();
        None
      } else {
        let n = self.expect_ident()?;
        self.expect(TokenKind::RParen)?;
        Some(n)
      }
    } else if requires_name {
      bail!("state(name) required inside flows");
    } else {
      None
    };
    self.consume_newlines();
    let statements = self.parse_state_statements(true)?;
    self.expect_keyword(Keyword::End)?;
    Ok(StateBlock { name, statements })
  }

  fn parse_state_statements(&mut self, allow_actions: bool) -> Result<Vec<StateStmt>> {
    let mut statements = Vec::new();
    while !self.check_keyword(Keyword::End) && !self.is_eof() {
      if self.check(TokenKind::Newline) {
        self.consume_newlines();
        continue;
      }
      if allow_actions && self.check_keyword(Keyword::On) {
        self.expect_keyword(Keyword::On)?;
        let event = self.parse_call()?;
        self.expect_transition()?;
        let target = self.expect_ident()?;
        statements.push(StateStmt::On { event, target });
      } else if allow_actions && self.check_keyword(Keyword::Run) {
        self.expect_keyword(Keyword::Run)?;
        let flow = self.expect_ident()?;
        statements.push(StateStmt::Run { flow });
      } else if allow_actions && self.check_keyword(Keyword::Terminate) {
        self.expect_keyword(Keyword::Terminate)?;
        statements.push(StateStmt::Terminate);
      } else {
        let stmt = self.parse_assignment_or_expr()?;
        statements.push(stmt);
      }
      self.consume_newlines();
    }
    Ok(statements)
  }

  fn parse_rule(&mut self) -> Result<Rule> {
    self.expect_keyword(Keyword::Rule)?;
    let (name, params) = self.parse_named_param_list()?;
    self.consume_newlines();

    let trigger = if self.check_keyword(Keyword::On) {
      self.expect_keyword(Keyword::On)?;
      RuleTrigger::On(self.parse_call()?)
    } else if self.check_keyword(Keyword::When) {
      self.expect_keyword(Keyword::When)?;
      RuleTrigger::When(self.parse_expr()?)
    } else {
      bail!("rule must start with on/when");
    };

    self.consume_newlines();
    let mut body = Vec::new();
    while !self.check_keyword(Keyword::End) && !self.is_eof() {
      if self.check(TokenKind::Newline) {
        self.consume_newlines();
        continue;
      }
      if self.check_keyword(Keyword::Emit) {
        self.expect_keyword(Keyword::Emit)?;
        let event = self.expect_ident()?;
        body.push(RuleStmt::Emit { event });
      } else {
        let stmt = self.parse_rule_assignment()?;
        body.push(stmt);
      }
      self.consume_newlines();
    }
    self.expect_keyword(Keyword::End)?; // end trigger
    self.consume_newlines();
    self.expect_keyword(Keyword::End)?; // end rule

    Ok(Rule { name, params, trigger, body })
  }

  fn parse_nd(&mut self) -> Result<NdBlock> {
    self.expect_keyword(Keyword::Nd)?;
    let (name, params) = self.parse_named_param_list()?;
    self.consume_newlines();

    self.expect_keyword(Keyword::Propose)?;
    let propose = self.parse_call()?;
    self.consume_newlines();

    self.expect_keyword(Keyword::Satisfy)?;
    self.expect(TokenKind::LParen)?;
    let mut constraints = Vec::new();
    if !self.check(TokenKind::RParen) {
      loop {
        constraints.push(self.parse_call()?);
        if self.check(TokenKind::Comma) {
          self.advance();
          continue;
        }
        break;
      }
    }
    self.expect(TokenKind::RParen)?;
    self.consume_newlines();
    self.expect_keyword(Keyword::End)?;

    Ok(NdBlock { name, params, propose, constraints })
  }

  fn parse_assignment_or_expr(&mut self) -> Result<StateStmt> {
    let checkpoint = self.pos;
    if let Some(TokenKind::Identifier(name)) = self.peek_kind().cloned() {
      if self.check_n(1, TokenKind::Eq) || self.check_n(1, TokenKind::PlusEq) {
        self.advance();
        let op = if self.check(TokenKind::Eq) {
          self.advance();
          AssignOp::Set
        } else {
          self.advance();
          AssignOp::Add
        };
        let value = self.parse_expr()?;
        return Ok(StateStmt::Assign { target: name, op, value });
      }
    }
    self.pos = checkpoint;
    let call = self.parse_command_call()?;
    Ok(StateStmt::Expr(call))
  }

  fn parse_rule_assignment(&mut self) -> Result<RuleStmt> {
    if let Some(TokenKind::Identifier(name)) = self.peek_kind().cloned() {
      if self.check_n(1, TokenKind::Eq) || self.check_n(1, TokenKind::PlusEq) {
        self.advance();
        let op = if self.check(TokenKind::Eq) {
          self.advance();
          AssignOp::Set
        } else {
          self.advance();
          AssignOp::Add
        };
        let value = self.parse_expr()?;
        return Ok(RuleStmt::Assign { target: name, op, value });
      }
    }
    bail!("rule body only supports assignments or emit");
  }

  fn parse_expr(&mut self) -> Result<Expr> {
    let mut left = self.parse_primary()?;
    if self.check(TokenKind::Gte) {
      self.advance();
      let right = self.parse_primary()?;
      left = Expr::Binary { left: Box::new(left), op: BinaryOp::Gte, right: Box::new(right) };
    }
    Ok(left)
  }

  fn parse_primary(&mut self) -> Result<Expr> {
    match self.peek_kind().cloned() {
      Some(TokenKind::Number(n)) => {
        self.advance();
        Ok(Expr::Number(n))
      }
      Some(TokenKind::String(s)) => {
        self.advance();
        Ok(Expr::String(s))
      }
      Some(TokenKind::Keyword(Keyword::Null)) => {
        self.advance();
        Ok(Expr::Null)
      }
      Some(TokenKind::Identifier(_)) => {
        let call_or_ident = self.parse_call_or_ident()?;
        Ok(call_or_ident)
      }
      Some(TokenKind::LParen) => {
        self.advance();
        let expr = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        Ok(expr)
      }
      _ => {
        bail!("Unexpected token in expression: {:?}", self.peek_kind());
      }
    }
  }

  fn parse_call_or_ident(&mut self) -> Result<Expr> {
    let name = self.expect_ident()?;
    if self.check(TokenKind::LParen) {
      let args = self.parse_arg_list()?;
      Ok(Expr::Call(Call { name, args }))
    } else {
      Ok(Expr::Ident(name))
    }
  }

  fn parse_call(&mut self) -> Result<Call> {
    let name = self.expect_ident()?;
    let args = if self.check(TokenKind::LParen) {
      self.parse_arg_list()?
    } else {
      Vec::new()
    };
    Ok(Call { name, args })
  }

  fn parse_command_call(&mut self) -> Result<Call> {
    let name = self.expect_ident()?;
    if self.check(TokenKind::LParen) {
      let args = self.parse_arg_list()?;
      return Ok(Call { name, args });
    }
    let mut args = Vec::new();
    while !self.check(TokenKind::Newline)
      && !self.check_keyword(Keyword::End)
      && !self.check_keyword(Keyword::On)
      && !self.check_keyword(Keyword::Run)
      && !self.check_keyword(Keyword::Terminate)
      && !self.check_keyword(Keyword::State)
      && !self.check_keyword(Keyword::Flow)
      && !self.check_keyword(Keyword::Rule)
      && !self.check_keyword(Keyword::Nd)
      && !self.check_keyword(Keyword::Start)
      && !self.is_eof() {
      let expr = self.parse_expr()?;
      args.push(CallArg { name: None, value: expr });
      if self.check(TokenKind::Comma) {
        self.advance();
      }
    }
    Ok(Call { name, args })
  }

  fn parse_named_param_list(&mut self) -> Result<(String, Vec<String>)> {
    self.expect(TokenKind::LParen)?;
    let name = self.expect_ident()?;
    let mut params = Vec::new();
    if self.check(TokenKind::Comma) {
      self.advance();
      loop {
        params.push(self.expect_ident()?);
        if self.check(TokenKind::Comma) {
          self.advance();
          continue;
        }
        break;
      }
    }
    self.expect(TokenKind::RParen)?;
    Ok((name, params))
  }

  fn parse_arg_list(&mut self) -> Result<Vec<CallArg>> {
    self.expect(TokenKind::LParen)?;
    let mut args = Vec::new();
    if !self.check(TokenKind::RParen) {
      loop {
        let arg = self.parse_arg()?;
        args.push(arg);
        if self.check(TokenKind::Comma) {
          self.advance();
          continue;
        }
        break;
      }
    }
    self.expect(TokenKind::RParen)?;
    Ok(args)
  }

  fn parse_arg(&mut self) -> Result<CallArg> {
    if let Some(TokenKind::Identifier(name)) = self.peek_kind().cloned() {
      if self.check_n(1, TokenKind::Colon) {
        self.advance();
        self.expect(TokenKind::Colon)?;
        let value = self.parse_expr()?;
        return Ok(CallArg { name: Some(name), value });
      }
    }
    let value = self.parse_expr()?;
    Ok(CallArg { name: None, value })
  }

  fn expect_ident(&mut self) -> Result<String> {
    match self.peek_kind().cloned() {
      Some(TokenKind::Identifier(name)) => {
        self.advance();
        Ok(name)
      }
      _ => bail!("Expected identifier"),
    }
  }

  fn expect_keyword(&mut self, kw: Keyword) -> Result<()> {
    match self.peek_kind() {
      Some(TokenKind::Keyword(k)) if *k == kw => {
        self.advance();
        Ok(())
      }
      _ => bail!("Expected keyword {:?}", kw),
    }
  }

  fn expect(&mut self, kind: TokenKind) -> Result<()> {
    if self.check(kind.clone()) {
      self.advance();
      Ok(())
    } else {
      bail!("Expected {:?}", kind)
    }
  }

  fn expect_transition(&mut self) -> Result<()> {
    if self.check(TokenKind::Gt) {
      self.advance();
      return Ok(());
    }
    bail!("Expected transition '>'");
  }

  fn check(&self, kind: TokenKind) -> bool {
    matches!(self.peek_kind(), Some(k) if *k == kind)
  }

  fn check_keyword(&self, kw: Keyword) -> bool {
    matches!(self.peek_kind(), Some(TokenKind::Keyword(k)) if *k == kw)
  }

  fn check_n(&self, n: usize, kind: TokenKind) -> bool {
    self.tokens.get(self.pos + n).map(|t| t.kind.clone()) == Some(kind)
  }

  fn advance(&mut self) {
    if self.pos < self.tokens.len() {
      self.pos += 1;
    }
  }

  fn peek(&self) -> Option<&Token> {
    self.tokens.get(self.pos)
  }

  fn peek_kind(&self) -> Option<&TokenKind> {
    self.peek().map(|t| &t.kind)
  }

  fn consume_newlines(&mut self) {
    while self.check(TokenKind::Newline) {
      self.advance();
    }
  }

  fn is_eof(&self) -> bool {
    matches!(self.peek_kind(), Some(TokenKind::Eof))
  }
}
