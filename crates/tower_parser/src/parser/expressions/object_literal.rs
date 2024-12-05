use bumpalo::collections::Vec;

use crate::{
  number::es_number_to_string,
  parser::{
    ast::{
      expression::Expression,
      object::{
        ObjectGetter, ObjectMethod, ObjectProperty, ObjectSetter, PropertyDefinition, PropertyName,
      },
    },
    error::{ParseError, ParseErrorCode},
    lexer::token::{Name, Token},
    required_token, syntax_err, Parser,
  },
};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_object_literal(&mut self) -> Result<Expression<'a>, ParseError> {
    let mut elements = Vec::<ObjectProperty<'a>>::new_in(&self.allocator);

    loop {
      let element = match &self.context.token {
        Token::RightCurlyBracket => {
          self.next_token()?;
          break;
        }
        Token::Asterisk => {
          self.next_token()?;
          let property = self.read_property_name()?;
          let parameters = self.read_formal_parameters()?;
          let body = self.read_function_body()?;
          let definition = ObjectMethod {
            r#async: false,
            body,
            generator: true,
            parameters,
            property,
          };

          ObjectProperty::Method(self.allocator.alloc(definition))
        }
        Token::Name(Name::Async) => {
          self.next_token()?;
          match &self.context.token {
            Token::Colon | Token::LeftParenthesis => {
              self.read_property_definition(PropertyName::Static("async"))?
            }
            Token::Asterisk if !self.context.line_terminator => {
              self.next_token()?;

              let property = self.read_property_name()?;
              let parameters = self.read_formal_parameters()?;
              let body = self.read_function_body()?;
              let definition = ObjectMethod {
                r#async: true,
                body,
                generator: true,
                parameters,
                property,
              };

              ObjectProperty::Method(self.allocator.alloc(definition))
            }
            _ => {
              if self.context.line_terminator {
                return Err(syntax_err!());
              }

              let property = self.read_property_name()?;
              let parameters = self.read_formal_parameters()?;
              let body = self.read_function_body()?;
              let definition = ObjectMethod {
                r#async: true,
                body,
                generator: false,
                parameters,
                property,
              };

              ObjectProperty::Method(self.allocator.alloc(definition))
            }
          }
        }
        Token::Name(Name::Get) => {
          self.next_token()?;
          match &self.context.token {
            Token::Colon | Token::LeftParenthesis => {
              self.read_property_definition(PropertyName::Static("get"))?
            }
            _ => {
              let property = self.read_property_name()?;
              required_token!(self, Token::LeftParenthesis);
              required_token!(self, Token::RightParenthesis);
              let body = self.read_function_body()?;
              let definition = ObjectGetter { property, body };

              ObjectProperty::Getter(self.allocator.alloc(definition))
            }
          }
        }
        Token::Name(Name::Set) => {
          self.next_token()?;
          match &self.context.token {
            Token::Colon | Token::LeftParenthesis => {
              self.read_property_definition(PropertyName::Static("set"))?
            }
            _ => {
              let property = self.read_property_name()?;
              required_token!(self, Token::LeftParenthesis);
              let parameter = self.read_binding_pattern_with_initializer()?;
              required_token!(self, Token::RightParenthesis);
              let body = self.read_function_body()?;
              let definition = ObjectSetter {
                body,
                parameter,
                property,
              };

              ObjectProperty::Setter(self.allocator.alloc(definition))
            }
          }
        }
        Token::Name(name) => {
          let name = name.clone();
          self.next_token()?;
          match &self.context.token {
            Token::Colon | Token::LeftParenthesis => {
              let property = PropertyName::Static(self.allocator.alloc_str(name.as_string()));
              self.read_property_definition(property)?
            }
            _ => match self.name_as_identifier_reference(&name)? {
              Some(expr) => ObjectProperty::Shorthand(expr),
              None => return Err(syntax_err!()),
            },
          }
        }
        _ => {
          let property = self.read_property_name()?;
          required_token!(self, Token::Colon);
          let definition = PropertyDefinition {
            property,
            expression: self.read_assignment_expression()?.ok_or(syntax_err!())?,
          };

          ObjectProperty::Property(self.allocator.alloc(definition))
        }
      };

      elements.push(element);

      match &self.context.token {
        Token::Comma => self.next_token()?,
        Token::RightCurlyBracket => {
          self.next_token()?;
          break;
        }
        _ => Err(syntax_err!())?,
      }
    }

    todo!()
  }

  fn read_property_name(&mut self) -> Result<PropertyName<'a>, ParseError> {
    let property = match &self.context.token {
      Token::Name(name) => {
        let name = self.allocator.alloc_str(name.as_string());
        self.next_token()?;
        PropertyName::Static(name)
      }
      Token::StringLiteral(string_literal) => {
        let name = self.allocator.alloc_str(&string_literal);
        self.next_token()?;
        PropertyName::Static(name)
      }
      Token::NumberLiteral(number_literal) => {
        let name = self
          .allocator
          .alloc_str(&es_number_to_string(*number_literal, 10));
        self.next_token()?;
        PropertyName::Static(name)
      }
      Token::BigIntLiteral(_) => {
        todo!()
      }
      Token::LeftSquareBracket => {
        self.next_token()?;
        let expression = self.read_assignment_expression()?.ok_or(syntax_err!())?;
        required_token!(self, Token::RightSquareBracket);
        PropertyName::Computed(expression)
      }
      _ => return Err(syntax_err!()),
    };

    Ok(property)
  }

  fn read_property_definition(
    &mut self,
    property: PropertyName<'a>,
  ) -> Result<ObjectProperty<'a>, ParseError> {
    let element = match &self.context.token {
      Token::Colon => {
        self.next_token()?;
        let definition = PropertyDefinition {
          property,
          expression: self.read_assignment_expression()?.ok_or(syntax_err!())?,
        };

        ObjectProperty::Property(self.allocator.alloc(definition))
      }
      Token::LeftParenthesis => {
        let parameters = self.read_formal_parameters()?;
        let body = self.read_function_body()?;
        let definition = ObjectMethod {
          r#async: false,
          body,
          generator: false,
          parameters,
          property,
        };

        ObjectProperty::Method(self.allocator.alloc(definition))
      }
      _ => return Err(syntax_err!()),
    };

    Ok(element)
  }
}
