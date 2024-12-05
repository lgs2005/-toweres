use bumpalo::{collections::Vec, vec};

use crate::parser::{
  ast::{
    binding::{BindingPattern, BindingPatternInitializer},
    expression::Expression,
    function::{Argument, ArrowFunctionDefinition, FormalParameters, FunctionDefinition},
    statement::Statement,
  },
  error::{ParseError, ParseErrorCode},
  lexer::token::Token,
  required_token, syntax_err, Parser,
};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_function_expression(&mut self, r#async: bool) -> Result<Expression<'a>, ParseError> {
    let generator = match &self.context.token {
      Token::Asterisk => {
        self.next_token()?;
        true
      }
      _ => false,
    };

    let identifier = match &self.context.token {
      Token::LeftParenthesis => None,
      _ => {
        let identifier = self.read_binding_identifier()?;
        if self.context.flags.strict_mode && matches!(identifier, "eval" | "arguments") {
          return Err(syntax_err!());
        }

        Some(identifier)
      }
    };

    let parameters = self.read_formal_parameters()?;
    let body = self.read_function_body()?;
    let definition = FunctionDefinition {
      r#async,
      body,
      generator,
      identifier,
      parameters,
    };

    Ok(Expression::Function(self.allocator.alloc(definition)))
  }

  pub fn read_formal_parameters(&mut self) -> Result<FormalParameters<'a>, ParseError> {
    required_token!(self, Token::LeftParenthesis);

    let mut bindings = Vec::<BindingPatternInitializer<'a>>::new_in(&self.allocator);
    let rest = loop {
      match &self.context.token {
        Token::RightParenthesis => {
          self.next_token()?;
          break None;
        }
        Token::TripleStop => {
          self.next_token()?;
          let pattern = self.read_binding_pattern()?;
          required_token!(self, Token::RightParenthesis);
          break Some(pattern);
        }
        _ => {
          let pattern = self.read_binding_pattern_with_initializer()?;
          bindings.push(pattern);

          match &self.context.token {
            Token::Comma => self.next_token()?,
            Token::RightParenthesis => {
              self.next_token()?;
              break None;
            }
            _ => return Err(syntax_err!()),
          }
        }
      }
    };

    Ok(FormalParameters { bindings, rest })
  }

  pub fn read_function_body(&mut self) -> Result<Vec<'a, Statement<'a>>, ParseError> {
    self.read_block_statement()
  }

  pub fn read_arguments(&mut self) -> Result<Vec<'a, Argument<'a>>, ParseError> {
    required_token!(self, Token::LeftParenthesis);
    let mut arguments = Vec::<Argument<'a>>::new_in(&self.allocator);

    loop {
      let argument = match &self.context.token {
        Token::RightParenthesis => {
          self.next_token()?;
          break;
        }
        Token::TripleStop => {
          self.next_token()?;
          let expression = self.read_assignment_expression()?.ok_or(syntax_err!())?;
          Argument::Spread(expression)
        }
        _ => {
          let expression = self.read_assignment_expression()?.ok_or(syntax_err!())?;
          Argument::Positional(expression)
        }
      };

      arguments.push(argument);

      match &self.context.token {
        Token::RightParenthesis => {
          self.next_token()?;
          break;
        }
        Token::Comma => {
          self.next_token()?;
        }
        _ => return Err(syntax_err!()),
      }
    }

    Ok(arguments)
  }

  pub fn read_arrow_function_expression(
    &mut self,
    r#async: bool,
  ) -> Result<Option<Expression<'a>>, ParseError> {
    let snapshot = self.context.clone();

    let parameters = match &self.context.token {
      Token::LeftParenthesis => match self.read_formal_parameters() {
        Ok(parameters) => parameters,
        Err(_) => {
          self.context = snapshot;
          return Ok(None);
        }
      },
      Token::Name(name) => match self.name_as_binding_identifier(name)? {
        Some(identifier) => {
          self.next_token()?;
          FormalParameters {
            bindings: vec![in &self.allocator; BindingPatternInitializer { initializer: None, pattern: BindingPattern::Identifier(identifier) }],
            rest: None,
          }
        }
        None => return Ok(None),
      },
      _ => return Ok(None),
    };

    if self.context.line_terminator || !matches!(self.context.token, Token::FatArrow) {
      self.context = snapshot;
      return Ok(None);
    }

    self.next_token()?;

    let body = match &self.context.token {
      Token::LeftSquareBracket => self.read_function_body()?,
      _ => {
        let _expression = self.read_assignment_expression()?;
        todo!()
      }
    };

    let definition = ArrowFunctionDefinition {
      r#async,
      body,
      parameters,
    };

    Ok(Some(Expression::ArrowFunction(
      self.allocator.alloc(definition),
    )))
  }
}
