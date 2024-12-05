use bumpalo::collections::Vec;

use crate::{
  number::es_number_to_string,
  parser::{
    ast::{
      class::{
        ClassDefinition, ClassElement, ClassElementName, ClassField, ClassGetter, ClassMethod,
        ClassSetter,
      },
      expression::Expression,
    },
    error::{ParseError, ParseErrorCode},
    lexer::token::{Name, Token},
    required_token, syntax_err, Parser,
  },
};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_class_expression(&mut self) -> Result<Expression<'a>, ParseError> {
    let identifier = match &self.context.token {
      Token::Name(Name::Extends) | Token::LeftCurlyBracket => None,
      _ => Some(self.read_binding_identifier()?),
    };

    let heritage = match &self.context.token {
      Token::LeftCurlyBracket => None,
      Token::Name(Name::Extends) => {
        self.next_token()?;
        Some(
          self
            .read_left_hand_side_expression()?
            .ok_or(syntax_err!())?,
        )
      }
      _ => return Err(syntax_err!()),
    };

    required_token!(self, Token::LeftCurlyBracket);

    let mut body = Vec::<ClassElement<'a>>::new_in(&self.allocator);

    loop {
      match &self.context.token {
        Token::RightCurlyBracket => {
          self.next_token()?;
          break;
        }
        Token::Semicolon => {
          self.next_token()?;
        }
        Token::Name(Name::Static) => {
          self.next_token()?;
          match &self.context.token {
            Token::LeftCurlyBracket => {
              todo!()
            }
            Token::Equals | Token::LeftParenthesis | Token::RightCurlyBracket => {
              body.push(self.read_field_definition(ClassElementName::Static("static"), false)?)
            }
            _ => body.push(self.read_field_or_method_definition(true)?),
          }
        }
        _ => body.push(self.read_field_or_method_definition(false)?),
      }
    }

    let definition = ClassDefinition {
      body,
      heritage,
      identifier,
    };

    Ok(Expression::Class(self.allocator.alloc(definition)))
  }

  fn read_field_or_method_definition(
    &mut self,
    r#static: bool,
  ) -> Result<ClassElement<'a>, ParseError> {
    let element = match &self.context.token {
      Token::Asterisk => {
        self.next_token()?;
        let name = self.read_element_name()?;
        let parameters = self.read_formal_parameters()?;
        let body = self.read_function_body()?;
        let definition = ClassMethod {
          r#async: false,
          body,
          generator: true,
          name,
          parameters,
          r#static,
        };

        ClassElement::Method(self.allocator.alloc(definition))
      }
      Token::Name(Name::Async) => {
        self.next_token()?;
        match &self.context.token {
          Token::Equals | Token::LeftParenthesis | Token::RightCurlyBracket => {
            self.read_field_definition(ClassElementName::Static("async"), r#static)?
          }
          Token::Asterisk if !self.context.line_terminator => {
            self.next_token()?;
            let name = self.read_element_name()?;
            let parameters = self.read_formal_parameters()?;
            let body = self.read_function_body()?;
            let definition = ClassMethod {
              r#async: true,
              body,
              generator: true,
              name,
              parameters,
              r#static,
            };

            ClassElement::Method(self.allocator.alloc(definition))
          }
          _ => {
            if self.context.line_terminator {
              return Err(syntax_err!());
            }

            let name = self.read_element_name()?;
            let parameters = self.read_formal_parameters()?;
            let body = self.read_function_body()?;
            let definition = ClassMethod {
              r#async: true,
              body,
              generator: false,
              name,
              parameters,
              r#static,
            };

            ClassElement::Method(self.allocator.alloc(definition))
          }
        }
      }
      Token::Name(Name::Get) => {
        self.next_token()?;
        match &self.context.token {
          Token::Equals | Token::LeftParenthesis | Token::RightCurlyBracket => {
            self.read_field_definition(ClassElementName::Static("get"), r#static)?
          }
          _ => {
            let name = self.read_element_name()?;
            required_token!(self, Token::LeftParenthesis);
            required_token!(self, Token::RightParenthesis);
            let body = self.read_function_body()?;
            let definition = ClassGetter {
              body,
              name,
              r#static,
            };

            ClassElement::Getter(self.allocator.alloc(definition))
          }
        }
      }
      Token::Name(Name::Set) => {
        self.next_token()?;
        match &self.context.token {
          Token::Equals | Token::LeftParenthesis | Token::RightCurlyBracket => {
            self.read_field_definition(ClassElementName::Static("set"), r#static)?
          }
          _ => {
            let name = self.read_element_name()?;
            required_token!(self, Token::LeftParenthesis);
            let parameter = self.read_binding_pattern_with_initializer()?;
            required_token!(self, Token::RightParenthesis);
            let body = self.read_function_body()?;
            let definition = ClassSetter {
              body,
              name,
              parameter,
              r#static,
            };

            ClassElement::Setter(self.allocator.alloc(definition))
          }
        }
      }
      _ => {
        let name = self.read_element_name()?;
        self.read_field_definition(name, r#static)?
      }
    };

    Ok(element)
  }

  fn read_element_name(&mut self) -> Result<ClassElementName<'a>, ParseError> {
    let name = match &self.context.token {
      Token::NumberSign => {
        self.next_token()?;
        match &self.context.token {
          Token::Name(name) => {
            let name = self.allocator.alloc_str(name.as_string());
            self.next_token()?;
            ClassElementName::Private(name)
          }
          _ => return Err(syntax_err!()),
        }
      }
      Token::Name(name) => {
        let name = self.allocator.alloc_str(name.as_string());
        self.next_token()?;
        ClassElementName::Static(name)
      }
      Token::StringLiteral(string_literal) => {
        let name = self.allocator.alloc_str(&string_literal);
        self.next_token()?;
        ClassElementName::Static(name)
      }
      Token::NumberLiteral(number_literal) => {
        let name = self
          .allocator
          .alloc_str(&es_number_to_string(*number_literal, 10));
        self.next_token()?;
        ClassElementName::Static(name)
      }
      Token::BigIntLiteral(_) => {
        todo!()
      }
      Token::LeftSquareBracket => {
        self.next_token()?;
        let expression = self.read_assignment_expression()?.ok_or(syntax_err!())?;
        required_token!(self, Token::RightSquareBracket);
        ClassElementName::Computed(expression)
      }
      _ => return Err(syntax_err!()),
    };

    Ok(name)
  }

  fn read_field_definition(
    &mut self,
    name: ClassElementName<'a>,
    r#static: bool,
  ) -> Result<ClassElement<'a>, ParseError> {
    let element = match &self.context.token {
      Token::Equals => {
        self.next_token()?;
        let value = self.read_assignment_expression()?.ok_or(syntax_err!())?;
        let definition = ClassField {
          name,
          r#static,
          value: Some(value),
        };

        self.auto_semicolon()?;

        ClassElement::Field(self.allocator.alloc(definition))
      }
      Token::LeftParenthesis => {
        self.next_token()?;
        let parameters = self.read_formal_parameters()?;
        let body = self.read_function_body()?;
        let definition = ClassMethod {
          r#async: false,
          body,
          generator: false,
          name,
          parameters,
          r#static,
        };

        ClassElement::Method(self.allocator.alloc(definition))
      }
      _ => {
        self.auto_semicolon()?;
        let definition = ClassField {
          name,
          r#static,
          value: None,
        };

        ClassElement::Field(self.allocator.alloc(definition))
      }
    };

    Ok(element)
  }
}
