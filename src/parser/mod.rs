use anyhow::{bail, Result};

use crate::ast::*;
use crate::lexer::{lex, Keyword, Token, TokenKind};
use std::collections::HashMap;

pub fn parse_source(input: &str) -> Result<Module> {
    let tokens = lex(input)?;
    let mut parser = Parser {
        tokens,
        pos: 0,
        anon_rule_counter: 0,
    };
    parser.parse_module()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    anon_rule_counter: usize,
}

impl Parser {
    fn parse_module(&mut self) -> Result<Module> {
        self.consume_newlines();
        let meta = self.parse_meta_headers()?;
        self.expect_keyword(Keyword::Module)?;
        self.expect(TokenKind::LParen)?;
        let name = self.parse_qualified_ident()?;
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Colon)?;
        let mut uses = Vec::new();
        let mut items = Vec::new();
        self.consume_newlines();

        while !self.check_keyword(Keyword::End) && !self.is_eof() {
            if self.check_keyword(Keyword::Use) {
                uses.push(self.parse_use()?);
            } else if self.check_keyword(Keyword::Flow) {
                items.push(Item::Flow(self.parse_flow()?));
            } else if self.check_keyword(Keyword::State) {
                items.push(Item::GlobalState(self.parse_global_state()?));
            } else if self.check_keyword(Keyword::Rule) {
                items.push(Item::Rule(self.parse_rule(None, None)?));
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
        Ok(Module {
            name,
            meta,
            uses,
            items,
        })
    }

    fn parse_use(&mut self) -> Result<UseDecl> {
        self.expect_keyword(Keyword::Use)?;
        self.expect(TokenKind::LParen)?;
        let path = self.parse_qualified_ident()?;
        let mut alias = None;
        if self.check(TokenKind::Comma) {
            self.advance();
            let key = self.expect_ident()?;
            if key != "as" {
                bail!("Expected 'as' in use(...) declaration");
            }
            self.expect(TokenKind::Colon)?;
            alias = Some(self.expect_ident()?);
        }
        self.expect(TokenKind::RParen)?;
        Ok(UseDecl { path, alias })
    }

    fn parse_meta_headers(&mut self) -> Result<HashMap<String, String>> {
        let mut meta = HashMap::new();
        loop {
            self.consume_newlines();
            if !self.check(TokenKind::At) {
                break;
            }
            self.expect(TokenKind::At)?;
            let tag = self.expect_ident()?;
            if tag != "meta" {
                bail!("Unknown directive @{}", tag);
            }

            while !self.check(TokenKind::Newline) && !self.is_eof() {
                let key = self.expect_ident()?.to_lowercase();
                self.expect(TokenKind::Eq)?;
                let value = self.expect_meta_value()?;
                meta.insert(key, value);

                if self.check(TokenKind::Comma) {
                    self.advance();
                }
            }
            self.consume_newlines();
        }
        Ok(meta)
    }

    fn expect_meta_value(&mut self) -> Result<String> {
        if let Some(tok) = self.peek() {
            match &tok.kind {
                TokenKind::String(s) => {
                    let out = s.clone();
                    self.advance();
                    Ok(out)
                }
                TokenKind::Number(n) => {
                    let out = n.to_string();
                    self.advance();
                    Ok(out)
                }
                TokenKind::Identifier(s) => {
                    let out = s.clone();
                    self.advance();
                    Ok(out)
                }
                TokenKind::Keyword(k) => {
                    let out = format!("{:?}", k).to_lowercase();
                    self.advance();
                    Ok(out)
                }
                _ => bail!("Expected meta value"),
            }
        } else {
            bail!("Expected meta value")
        }
    }

    fn parse_flow(&mut self) -> Result<Flow> {
        self.expect_keyword(Keyword::Flow)?;
        self.expect(TokenKind::LParen)?;
        let name = self.parse_qualified_ident()?;
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Colon)?;
        let mut start = None;
        let mut states = Vec::new();
        self.consume_newlines();

        while !self.check_keyword(Keyword::End) && !self.is_eof() {
            if self.check_keyword(Keyword::Start) {
                self.expect_keyword(Keyword::Start)?;
                self.expect_transition()?;
                let target = self.parse_qualified_ident()?;
                start = Some(target);
            } else if self.check_keyword(Keyword::State) {
                states.push(self.parse_state_block(true, Some(name.as_str()))?);
            } else if self.check(TokenKind::Newline) {
                self.consume_newlines();
            } else {
                let tok = self.peek().cloned();
                bail!("Unexpected token {:?} in flow", tok.map(|t| t.kind));
            }
            self.consume_newlines();
        }

        self.expect_keyword(Keyword::End)?;
        Ok(Flow {
            name,
            start,
            states,
        })
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
        self.expect(TokenKind::Colon)?;
        self.consume_newlines();
        let statements = self.parse_state_statements(false, None, None)?;
        self.expect_keyword(Keyword::End)?;
        Ok(StateBlock {
            name: None,
            statements,
        })
    }

    fn parse_state_block(
        &mut self,
        requires_name: bool,
        flow_name: Option<&str>,
    ) -> Result<StateBlock> {
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
        self.expect(TokenKind::Colon)?;
        self.consume_newlines();
        let statements = self.parse_state_statements(true, flow_name, name.as_deref())?;
        self.expect_keyword(Keyword::End)?;
        Ok(StateBlock { name, statements })
    }

    fn parse_state_statements(
        &mut self,
        allow_actions: bool,
        flow_name: Option<&str>,
        state_name: Option<&str>,
    ) -> Result<Vec<StateStmt>> {
        let mut statements = Vec::new();
        while !self.check_keyword(Keyword::End) && !self.is_eof() {
            if self.check(TokenKind::Newline) {
                self.consume_newlines();
                continue;
            }
            if allow_actions && self.check_keyword(Keyword::On) {
                self.expect_keyword(Keyword::On)?;
                let event = self.parse_call()?;
                if self.check(TokenKind::Gt) {
                    self.expect_transition()?;
                    let target = self.parse_qualified_ident()?;
                    statements.push(StateStmt::On { event, target });
                } else if self.check(TokenKind::DoubleColon) {
                    self.advance();
                    let body_stmt = self.parse_rule_stmt()?;
                    statements.push(StateStmt::Rule(self.build_inline_on_rule(
                        event,
                        vec![body_stmt],
                        flow_name,
                        state_name,
                    )));
                } else if self.check(TokenKind::Colon) {
                    self.expect(TokenKind::Colon)?;
                    self.consume_newlines();
                    let mut body = Vec::new();
                    while !self.check_keyword(Keyword::End) && !self.is_eof() {
                        if self.check(TokenKind::Newline) {
                            self.consume_newlines();
                            continue;
                        }
                        body.push(self.parse_rule_stmt()?);
                        self.consume_newlines();
                    }
                    self.expect_keyword(Keyword::End)?;
                    statements.push(StateStmt::Rule(
                        self.build_inline_on_rule(event, body, flow_name, state_name),
                    ));
                } else {
                    bail!("Expected '>' or ':' or '::' after on-event");
                }
            } else if allow_actions && self.check_keyword(Keyword::Rule) {
                statements.push(StateStmt::Rule(self.parse_rule(flow_name, state_name)?));
            } else if allow_actions && self.check_keyword(Keyword::Run) {
                self.expect_keyword(Keyword::Run)?;
                let flow = self.parse_qualified_ident()?;
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

    fn parse_rule(&mut self, scope_flow: Option<&str>, scope_state: Option<&str>) -> Result<Rule> {
        self.expect_keyword(Keyword::Rule)?;
        let (name, params) = self.parse_named_param_list()?;
        self.expect(TokenKind::Colon)?;
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
        let body = if self.check(TokenKind::DoubleColon) {
            self.advance();
            vec![self.parse_rule_stmt()?]
        } else {
            self.expect(TokenKind::Colon)?;
            self.consume_newlines();
            let mut body = Vec::new();
            while !self.check_keyword(Keyword::End) && !self.is_eof() {
                if self.check(TokenKind::Newline) {
                    self.consume_newlines();
                    continue;
                }
                body.push(self.parse_rule_stmt()?);
                self.consume_newlines();
            }
            self.expect_keyword(Keyword::End)?; // end trigger
            body
        };

        self.consume_newlines();
        self.expect_keyword(Keyword::End)?; // end rule

        Ok(Rule {
            name,
            params,
            scope_flow: scope_flow.map(str::to_string),
            scope_state: scope_state.map(str::to_string),
            trigger,
            body,
        })
    }

    fn parse_nd(&mut self) -> Result<NdBlock> {
        self.expect_keyword(Keyword::Nd)?;
        let (name, params) = self.parse_named_param_list()?;
        self.expect(TokenKind::Colon)?;
        self.consume_newlines();

        self.expect_keyword(Keyword::Propose)?;
        let propose = self.parse_call()?;
        self.consume_newlines();

        self.expect_keyword(Keyword::Satisfy)?;
        self.expect(TokenKind::LParen)?;
        let mut constraints = Vec::new();
        self.consume_newlines();
        if !self.check(TokenKind::RParen) {
            loop {
                self.consume_newlines();
                constraints.push(self.parse_call()?);
                self.consume_newlines();
                if self.check(TokenKind::Comma) {
                    self.advance();
                    self.consume_newlines();
                    continue;
                }
                break;
            }
        }
        self.expect(TokenKind::RParen)?;
        self.consume_newlines();
        self.expect_keyword(Keyword::End)?;

        Ok(NdBlock {
            name,
            params,
            propose,
            constraints,
        })
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
                return Ok(StateStmt::Assign {
                    target: name,
                    op,
                    value,
                });
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
                return Ok(RuleStmt::Assign {
                    target: name,
                    op,
                    value,
                });
            }
        }
        bail!("rule body only supports assignments or emit");
    }

    fn parse_rule_stmt(&mut self) -> Result<RuleStmt> {
        if self.check_keyword(Keyword::Emit) {
            self.expect_keyword(Keyword::Emit)?;
            let event = self.parse_qualified_ident()?;
            return Ok(RuleStmt::Emit { event });
        }
        self.parse_rule_assignment()
    }

    fn build_inline_on_rule(
        &mut self,
        event: Call,
        body: Vec<RuleStmt>,
        scope_flow: Option<&str>,
        scope_state: Option<&str>,
    ) -> Rule {
        self.anon_rule_counter += 1;
        let line = self.peek().map(|t| t.line).unwrap_or(0);
        let mut name = format!("__on_{}_{}", line, self.anon_rule_counter);
        if let Some(state) = scope_state {
            name.push('_');
            name.push_str(state);
        }
        Rule {
            name,
            params: Vec::new(),
            scope_flow: scope_flow.map(str::to_string),
            scope_state: scope_state.map(str::to_string),
            trigger: RuleTrigger::On(event),
            body,
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_and_expr()?;
        while self.check_keyword(Keyword::Or) {
            self.expect_keyword(Keyword::Or)?;
            let right = self.parse_and_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison_expr()?;
        while self.check_keyword(Keyword::And) {
            self.expect_keyword(Keyword::And)?;
            let right = self.parse_comparison_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_primary()?;
        if self.check(TokenKind::Gte)
            || self.check(TokenKind::Gt)
            || self.check(TokenKind::Lt)
            || self.check(TokenKind::EqEq)
            || self.check(TokenKind::Neq)
        {
            let op = if self.check(TokenKind::Gte) {
                self.advance();
                BinaryOp::Gte
            } else if self.check(TokenKind::Gt) {
                self.advance();
                BinaryOp::Gt
            } else if self.check(TokenKind::Lt) {
                self.advance();
                BinaryOp::Lt
            } else if self.check(TokenKind::EqEq) {
                self.expect(TokenKind::EqEq)?;
                BinaryOp::Eq
            } else {
                self.expect(TokenKind::Neq)?;
                BinaryOp::Neq
            };
            let right = self.parse_primary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
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
            Some(TokenKind::Question) => {
                self.advance();
                let ident = self.expect_ident()?;
                Ok(Expr::Ident(format!("?{}", ident)))
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
        let name = self.parse_qualified_ident()?;
        if self.check(TokenKind::LParen) {
            let args = self.parse_arg_list()?;
            Ok(Expr::Call(Call { name, args }))
        } else {
            Ok(Expr::Ident(name))
        }
    }

    fn parse_call(&mut self) -> Result<Call> {
        let name = self.parse_qualified_ident()?;
        let args = if self.check(TokenKind::LParen) {
            self.parse_arg_list()?
        } else {
            Vec::new()
        };
        Ok(Call { name, args })
    }

    fn parse_command_call(&mut self) -> Result<Call> {
        let name = self.parse_qualified_ident()?;
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
            && !self.is_eof()
        {
            let expr = self.parse_expr()?;
            args.push(CallArg {
                name: None,
                value: expr,
            });
            if self.check(TokenKind::Comma) {
                self.advance();
            }
        }
        Ok(Call { name, args })
    }

    fn parse_named_param_list(&mut self) -> Result<(String, Vec<String>)> {
        self.expect(TokenKind::LParen)?;
        let name = self.parse_qualified_ident()?;
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

    fn parse_qualified_ident(&mut self) -> Result<String> {
        let mut name = self.expect_ident()?;
        while self.check(TokenKind::Dot) {
            self.advance();
            let segment = self.expect_ident()?;
            name.push('.');
            name.push_str(&segment);
        }
        Ok(name)
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
                return Ok(CallArg {
                    name: Some(name),
                    value,
                });
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
