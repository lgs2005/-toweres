use bumpalo::collections::Vec;

use crate::parser::{
  ast::expression::{Expression, TaggedTemplateLiteral, TemplateLiteral},
  error::{ParseError, ParseErrorCode},
  lexer::token::Token,
  parse_err, syntax_err, Parser,
};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_template_literal(
    &mut self,
    baked_string: Option<String>,
  ) -> Result<Expression<'a>, ParseError> {
    let mut strings = Vec::<&'a str>::new_in(&self.allocator);
    let mut substitutions = Vec::<Expression<'a>>::new_in(&self.allocator);

    let head_string = baked_string.ok_or(parse_err!(ParseErrorCode::InvalidTemplateString))?;
    strings.push(self.allocator.alloc_str(&head_string));

    loop {
      let expression = self.read_expression()?.ok_or(syntax_err!())?;
      substitutions.push(expression);

      match &self.context.token {
        Token::TemplateMiddle(_, baked_string) => {
          let baked_string = baked_string.clone();
          self.next_token()?;
          let string = baked_string.ok_or(parse_err!(ParseErrorCode::InvalidTemplateString))?;
          strings.push(self.allocator.alloc_str(&string));
        }
        Token::TemplateTail(_, baked_string) => {
          let baked_string = baked_string.clone();
          self.next_token()?;
          let string = baked_string.ok_or(parse_err!(ParseErrorCode::InvalidTemplateString))?;
          strings.push(self.allocator.alloc_str(&string));
          break;
        }
        _ => return Err(syntax_err!()),
      }
    }

    let literal = TemplateLiteral {
      strings,
      substitutions,
    };

    Ok(Expression::Template(self.allocator.alloc(literal)))
  }

  pub fn read_tagged_template_literal(
    &mut self,
    tag: Expression<'a>,
    optional: bool,
  ) -> Result<Expression<'a>, ParseError> {
    let mut strings = Vec::<Option<&'a str>>::new_in(&self.allocator);
    let mut substitutions = Vec::<Expression<'a>>::new_in(&self.allocator);
    let mut raw_strings = Vec::<&'a str>::new_in(&self.allocator);

    match &self.context.token {
      Token::TemplateHead(raw_string, baked_string) => {
        let raw_string = self.allocator.alloc_str(&raw_string);
        let baked_string = match baked_string {
          Some(string) => Some(self.allocator.alloc_str(&string) as &'a str),
          None => None,
        };

        self.next_token()?;
        strings.push(baked_string);
        raw_strings.push(raw_string);
      }
      _ => return Err(syntax_err!()),
    };

    loop {
      let expression = self.read_expression()?.ok_or(syntax_err!())?;
      substitutions.push(expression);

      match &self.context.token {
        Token::TemplateMiddle(raw_string, baked_string) => {
          let raw_string = self.allocator.alloc_str(&raw_string);
          let baked_string = match baked_string {
            Some(string) => Some(self.allocator.alloc_str(&string) as &'a str),
            None => None,
          };

          self.next_token()?;
          strings.push(baked_string);
          raw_strings.push(raw_string);
        }
        Token::TemplateTail(raw_string, baked_string) => {
          let raw_string = self.allocator.alloc_str(&raw_string);
          let baked_string = match baked_string {
            Some(string) => Some(self.allocator.alloc_str(&string) as &'a str),
            None => None,
          };

          self.next_token()?;
          strings.push(baked_string);
          raw_strings.push(raw_string);
          break;
        }
        _ => return Err(syntax_err!()),
      }
    }

    let literal = TaggedTemplateLiteral {
      optional,
      raw_strings,
      strings,
      substitutions,
      tag,
    };

    Ok(Expression::TaggedTemplate(self.allocator.alloc(literal)))
  }
}
