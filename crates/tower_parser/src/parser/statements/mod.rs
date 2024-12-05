use bumpalo::collections::Vec;

use crate::parser::ast::statement::{
  CatchBlock, DoWhileStatement, IfStatement, SwitchCase, SwitchStatement, WhileStatement,
  WithStatement,
};

use super::{
  ast::{
    binding::{BindingPattern, BindingPatternInitializer},
    statement::{LabelStatement, Statement, TryStatement},
  },
  error::{ParseError, ParseErrorCode},
  lexer::token::{Name, Token},
  required_token, syntax_err, Parser,
};

mod label;

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_statement(&mut self) -> Result<Option<Statement<'a>>, ParseError> {
    let statement = match &self.context.token {
      Token::LeftCurlyBracket => {
        let list = self.read_block_statement()?;
        Statement::Block(self.allocator.alloc(list))
      }
      Token::Name(Name::Var) => self.read_variable_statement()?,
      Token::Semicolon => {
        self.next_token()?;
        Statement::Empty
      }
      Token::Name(Name::If) => self.read_if_statement()?,
      Token::Name(Name::Do) => self.read_do_while_statement()?,
      Token::Name(Name::While) => self.read_while_statement()?,
      Token::Name(Name::For) => self.read_for_statement()?,
      Token::Name(Name::Switch) => self.read_switch_statement()?,
      Token::Name(Name::Continue) => self.read_continue_statement()?,
      Token::Name(Name::Break) => self.read_break_statement()?,
      Token::Name(Name::Return) => self.read_return_statement()?,
      Token::Name(Name::With) => self.read_with_statement()?,
      Token::Name(Name::Throw) => self.read_throw_statement()?,
      Token::Name(Name::Try) => self.read_try_statement()?,
      Token::Name(Name::Debugger) => {
        self.next_token()?;
        self.auto_semicolon()?;
        Statement::Debugger
      }
      Token::Name(Name::Function | Name::Class) => return Ok(None),
      Token::Name(Name::Async) => {
        let snapshot = self.context.clone();
        self.next_token()?;
        if !self.context.line_terminator
          && matches!(self.context.token, Token::Name(Name::Function))
        {
          self.context = snapshot;
          return Ok(None);
        } else {
          self.context = snapshot;
          return self.read_expression_statement();
        }
      }
      Token::Name(Name::Let) => {
        let snapshot = self.context.clone();
        self.next_token()?;
        if matches!(self.context.token, Token::LeftSquareBracket) {
          self.context = snapshot;
          return Ok(None);
        } else {
          self.context = snapshot;
          return self.read_expression_statement();
        }
      }
      Token::Name(name) => match self.name_as_label_identifier(name)? {
        Some(label) => {
          let snapshot = self.context.clone();
          self.next_token()?;
          match &self.context.token {
            Token::Colon => {
              self.next_token()?;
              let statement = match self.read_statement()? {
                Some(st) => st,
                None => todo!(),
              };

              Statement::Label(self.allocator.alloc(LabelStatement { label, statement }))
            }
            _ => {
              self.context = snapshot;
              return self.read_expression_statement();
            }
          }
        }
        None => return self.read_expression_statement(),
      },
      _ => return self.read_expression_statement(),
    };

    Ok(Some(statement))
  }

  fn read_expression_statement(&mut self) -> Result<Option<Statement<'a>>, ParseError> {
    match self.read_expression()? {
      Some(expr) => Ok(Some(Statement::Expression(self.allocator.alloc(expr)))),
      None => Ok(None),
    }
  }

  pub fn read_block_statement(&mut self) -> Result<Vec<'a, Statement<'a>>, ParseError> {
    required_token!(self, Token::LeftCurlyBracket);
    let mut list = Vec::<Statement<'a>>::new_in(&self.allocator);

    loop {
      match &self.context.token {
        Token::RightCurlyBracket => {
          self.next_token()?;
          break;
        }
        _ => {
          let statement = self.read_statement()?.ok_or(syntax_err!())?;
          list.push(statement);
        }
      }
    }

    Ok(list)
  }

  fn read_variable_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    let mut declarations = Vec::<BindingPatternInitializer<'a>>::new_in(&self.allocator);

    loop {
      let pattern = self.read_binding_pattern_with_initializer()?;

      if !matches!(pattern.pattern, BindingPattern::Identifier(_)) && pattern.initializer.is_none()
      {
        return Err(syntax_err!());
      }

      declarations.push(pattern);

      match &self.context.token {
        Token::Comma => self.next_token()?,
        _ => break,
      }
    }

    self.auto_semicolon()?;
    Ok(Statement::Variable(self.allocator.alloc(declarations)))
  }

  fn read_if_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    required_token!(self, Token::LeftParenthesis);
    let condition = self.read_expression()?.ok_or(syntax_err!())?;
    required_token!(self, Token::RightParenthesis);
    let consequent = self.read_statement()?.ok_or(syntax_err!())?;

    let alternate = match &self.context.token {
      Token::Name(Name::Else) => {
        self.next_token()?;
        Some(self.read_statement()?.ok_or(syntax_err!())?)
      }
      _ => None,
    };

    let statement = IfStatement {
      alternate,
      condition,
      consequent,
    };

    Ok(Statement::If(self.allocator.alloc(statement)))
  }

  fn read_do_while_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    let body = self.read_statement()?.ok_or(syntax_err!())?;
    required_token!(self, Token::Name(Name::While));
    required_token!(self, Token::LeftParenthesis);
    let condition = self.read_expression()?.ok_or(syntax_err!())?;
    required_token!(self, Token::RightParenthesis);

    match &self.context.token {
      Token::Semicolon => self.next_token()?,
      _ => {}
    }

    let statement = DoWhileStatement { body, condition };
    Ok(Statement::DoWhile(self.allocator.alloc(statement)))
  }

  fn read_while_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    required_token!(self, Token::LeftParenthesis);
    let condition = self.read_expression()?.ok_or(syntax_err!())?;
    required_token!(self, Token::RightParenthesis);
    let body = self.read_statement()?.ok_or(syntax_err!())?;
    let statement = WhileStatement { body, condition };
    Ok(Statement::While(self.allocator.alloc(statement)))
  }

  fn read_for_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    todo!()
  }

  fn read_switch_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    required_token!(self, Token::LeftParenthesis);
    let expression = self.read_expression()?.ok_or(syntax_err!())?;
    required_token!(self, Token::RightParenthesis);
    required_token!(self, Token::LeftCurlyBracket);

    let mut cases = Vec::<SwitchCase<'a>>::new_in(&self.allocator);

    loop {
      let expression = match &self.context.token {
        Token::Name(Name::Case) => {
          self.next_token()?;
          let expression = self.read_expression()?.ok_or(syntax_err!())?;
          required_token!(self, Token::Colon);
          Some(expression)
        }
        Token::Name(Name::Default) => {
          self.next_token()?;
          required_token!(self, Token::Colon);
          None
        }
        Token::RightCurlyBracket => {
          self.next_token()?;
          break;
        }
        _ => return Err(syntax_err!()),
      };

      let mut body = Vec::<Statement<'a>>::new_in(&self.allocator);

      loop {
        match &self.context.token {
          Token::RightCurlyBracket | Token::Name(Name::Default | Name::Case) => break,
          _ => {
            let statement = self.read_statement()?.ok_or(syntax_err!())?;
            body.push(statement);
          }
        }
      }

      let case = SwitchCase { body, expression };
      cases.push(case);
    }

    let statement = SwitchStatement { cases, expression };
    Ok(Statement::Switch(self.allocator.alloc(statement)))
  }

  fn read_return_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    let expression = if self.context.line_terminator {
      None
    } else {
      self.read_expression()?
    };
    self.auto_semicolon()?;
    Ok(Statement::Return(self.allocator.alloc(expression)))
  }

  fn read_with_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    required_token!(self, Token::LeftParenthesis);
    let expression = self.read_expression()?.ok_or(syntax_err!())?;
    required_token!(self, Token::RightParenthesis);
    let body = self.read_statement()?.ok_or(syntax_err!())?;
    let statement = WithStatement { body, expression };
    Ok(Statement::With(self.allocator.alloc(statement)))
  }

  fn read_throw_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    if self.context.line_terminator {
      return Err(syntax_err!());
    }
    let expression = self.read_expression()?.ok_or(syntax_err!())?;
    self.auto_semicolon()?;
    Ok(Statement::Throw(self.allocator.alloc(expression)))
  }

  fn read_try_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    let body = self.read_block_statement()?;

    let catch = match &self.context.token {
      Token::Name(Name::Catch) => {
        self.next_token()?;

        let parameter = match &self.context.token {
          Token::LeftParenthesis => {
            self.next_token()?;
            let pattern = self.read_binding_pattern()?;
            required_token!(self, Token::RightParenthesis);
            Some(pattern)
          }
          _ => None,
        };

        let body = self.read_block_statement()?;
        Some(CatchBlock { body, parameter })
      }
      _ => None,
    };

    let finally = match &self.context.token {
      Token::Name(Name::Finally) => {
        self.next_token()?;
        Some(self.read_block_statement()?)
      }
      _ => None,
    };

    let statement = TryStatement {
      body,
      catch,
      finally,
    };

    Ok(Statement::Try(self.allocator.alloc(statement)))
  }
}
