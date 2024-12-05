use bumpalo::collections::Vec;

use crate::{
  bigint::BigInt,
  parser::{
    ast::expression::{ArrayElement, Expression, RegExpLiteral},
    error::{ParseError, ParseErrorCode},
    lexer::token::{Name, Token},
    parse_err, required_token, syntax_err, Parser,
  },
};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_primary_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    let token = match &self.context.token {
      Token::Name(Name::This) => {
        self.next_token()?;
        Expression::This
      }
      Token::Name(Name::Null) => {
        self.next_token()?;
        Expression::Null
      }
      Token::Name(Name::True) => {
        self.next_token()?;
        Expression::Boolean(true)
      }
      Token::Name(Name::False) => {
        self.next_token()?;
        Expression::Boolean(false)
      }
      Token::NumberLiteral(numeric_literal) => {
        let value: &'a f64 = self.allocator.alloc(*numeric_literal);
        self.next_token()?;
        Expression::Number(value)
      }
      Token::BigIntLiteral(bigint_literal) => {
        let value: &'a BigInt = self.allocator.alloc(bigint_literal.clone());
        self.next_token()?;
        Expression::BigInt(value)
      }
      Token::StringLiteral(string_literal) => {
        let value: &'a str = self.allocator.alloc_str(string_literal.as_str());
        self.next_token()?;
        Expression::String(value)
      }
      Token::LeftSquareBracket => {
        self.next_token()?;
        self.read_array_literal()?
      }
      Token::LeftCurlyBracket => {
        self.next_token()?;
        self.read_object_literal()?
      }
      Token::Name(Name::Function) => {
        self.next_token()?;
        self.read_function_expression(false)?
      }
      Token::Name(Name::Class) => {
        self.next_token()?;
        self.read_class_expression()?
      }
      Token::Asterisk => {
        self.next_token()?;
        todo!()
      }
      Token::Name(Name::Async) => {
        self.next_token()?;
        if self.context.line_terminator
          || !matches!(self.context.token, Token::Name(Name::Function))
        {
          return Err(syntax_err!());
        }
        self.next_token()?;
        self.read_function_expression(true)?
      }
      Token::RegExp(source, flags) => {
        let literal = RegExpLiteral {
          source: self.allocator.alloc_str(&source),
          flags: self.allocator.alloc_str(&flags),
        };

        self.next_token()?;
        Expression::RegExp(self.allocator.alloc(literal))
      }
      Token::NoSubstitutionTemplate(_, baked_string) => match baked_string {
        Some(string) => Expression::String(self.allocator.alloc_str(string)),
        None => return Err(parse_err!(ParseErrorCode::InvalidTemplateString)),
      },
      Token::TemplateHead(_, baked_string) => {
        let baked_string = baked_string.clone();
        self.next_token()?;
        self.read_template_literal(baked_string)?
      }
      Token::LeftParenthesis => {
        self.next_token()?;
        let expression = self.read_expression()?.ok_or(syntax_err!())?;
        required_token!(self, Token::RightParenthesis);
        Expression::Group(self.allocator.alloc(expression))
      }
      _ => return self.read_identifier_reference(),
    };

    Ok(Some(token))
  }

  fn read_array_literal(&mut self) -> Result<Expression<'a>, ParseError> {
    let mut elements = Vec::new_in(&self.allocator);

    loop {
      match &self.context.token {
        Token::RightSquareBracket => {
          self.next_token()?;
          break;
        }
        Token::Comma => {
          self.next_token()?;
          elements.push(ArrayElement::Elision);
        }
        Token::TripleStop => {
          self.next_token()?;
          let expression = self.read_assignment_expression()?.ok_or(syntax_err!())?;
          required_token!(self, Token::Comma);
          elements.push(ArrayElement::Spread(expression));
        }
        _ => {
          let expression = self.read_assignment_expression()?.ok_or(syntax_err!())?;
          required_token!(self, Token::Comma);
          elements.push(ArrayElement::Expression(expression));
        }
      }
    }

    let token = Expression::Array(self.allocator.alloc(elements));

    Ok(token)
  }
}
