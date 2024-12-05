use crate::parser::{
  ast::{statement::Statement, SourceType},
  lexer::token::{Name, Token},
  Parser,
};

use super::{syntax_err, ParseError, ParseErrorCode};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_continue_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    let label = self.read_label_identifier()?;
    self.auto_semicolon()?;
    Ok(Statement::Continue(self.allocator.alloc(label)))
  }

  pub fn read_break_statement(&mut self) -> Result<Statement<'a>, ParseError> {
    self.next_token()?;
    let label = self.read_label_identifier()?;
    self.auto_semicolon()?;
    Ok(Statement::Break(self.allocator.alloc(label)))
  }

  fn read_label_identifier(&mut self) -> Result<Option<&'a str>, ParseError> {
    if self.context.line_terminator {
      Ok(None)
    } else {
      match &self.context.token {
        Token::Name(name) => self.name_as_label_identifier(name),
        _ => Ok(None),
      }
    }
  }

  pub fn name_as_label_identifier(&self, name: &Name) -> Result<Option<&'a str>, ParseError> {
    let string = match name {
      Name::Yield if !self.context.flags.param_yield => {
        if self.context.flags.strict_mode {
          return Err(syntax_err!());
        }

        Some("yield")
      }
      Name::Await if !self.context.flags.param_await => {
        if matches!(self.source_type, SourceType::Module) {
          return Err(syntax_err!());
        }

        Some("await")
      }
      name => match self.name_as_identifier(name)? {
        None => None,
        Some(string) => {
          if (self.context.flags.param_await && string == "await")
            || (self.context.flags.param_yield && string == "yield")
          {
            return Err(syntax_err!());
          }

          Some(string)
        }
      },
    };

    Ok(string)
  }
}
