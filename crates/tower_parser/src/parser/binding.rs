use bumpalo::collections::Vec;

use crate::number::es_number_to_string;

use super::{
  ast::{
    binding::{
      ArrayBindingPattern, BindingPattern, BindingPatternInitializer, ObjectBindingPattern,
      ObjectBindingProperty,
    },
    object::PropertyName,
    SourceType,
  },
  error::{syntax_err, ParseError, ParseErrorCode},
  lexer::token::{Name, Token},
  required_token, Parser,
};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_binding_pattern_with_initializer(
    &mut self,
  ) -> Result<BindingPatternInitializer<'a>, ParseError> {
    let pattern = self.read_binding_pattern()?;
    let initializer = match &self.context.token {
      Token::Equals => {
        self.next_token()?;
        Some(self.read_assignment_expression()?.ok_or(syntax_err!())?)
      }
      _ => None,
    };

    let pattern_with_initializer = BindingPatternInitializer {
      pattern,
      initializer,
    };

    Ok(pattern_with_initializer)
  }

  pub fn read_binding_pattern(&mut self) -> Result<BindingPattern<'a>, ParseError> {
    let pattern = match &self.context.token {
      Token::LeftSquareBracket => {
        self.next_token()?;
        let mut elements = Vec::<Option<BindingPatternInitializer<'a>>>::new_in(&self.allocator);
        let rest = loop {
          match &self.context.token {
            Token::Comma => {
              self.next_token()?;
              elements.push(None);
            }
            Token::TripleStop => {
              self.next_token()?;
              break Some(self.read_binding_pattern()?);
            }
            Token::RightSquareBracket => {
              break None;
            }
            _ => {
              let pattern = self.read_binding_pattern_with_initializer()?;
              elements.push(Some(pattern));

              match &self.context.token {
                Token::Comma => self.next_token()?,
                Token::RightSquareBracket => break None,
                _ => return Err(syntax_err!()),
              }
            }
          }
        };

        required_token!(self, Token::RightSquareBracket);
        BindingPattern::Array(self.allocator.alloc(ArrayBindingPattern { elements, rest }))
      }
      Token::LeftCurlyBracket => {
        self.next_token()?;

        if matches!(self.context.token, Token::RightCurlyBracket) {
          self.next_token()?;
          return Ok(BindingPattern::Object(self.allocator.alloc(
            ObjectBindingPattern {
              properties: Vec::new_in(&self.allocator),
              rest: None,
            },
          )));
        }

        let mut properties = Vec::<ObjectBindingProperty<'a>>::new_in(&self.allocator);
        let rest = loop {
          match &self.context.token {
            Token::Name(name) => {
              let name = name.clone();
              self.next_token()?;

              match &self.context.token {
                Token::Colon => {
                  self.next_token()?;

                  let binding = self.read_binding_pattern_with_initializer()?;
                  let property = ObjectBindingProperty {
                    property: PropertyName::Static(self.allocator.alloc_str(name.as_string())),
                    binding,
                  };

                  properties.push(property);
                }
                token => match self.name_as_binding_identifier(&name)? {
                  Some(identifier) => {
                    let initializer = match token {
                      Token::Equals => {
                        self.next_token()?;
                        Some(self.read_expression()?.ok_or(syntax_err!())?)
                      }
                      _ => None,
                    };

                    let property = ObjectBindingProperty {
                      property: PropertyName::Static(identifier),
                      binding: BindingPatternInitializer {
                        pattern: BindingPattern::Identifier(identifier),
                        initializer,
                      },
                    };

                    properties.push(property);
                  }
                  None => return Err(syntax_err!()),
                },
              }
            }
            Token::StringLiteral(string_literal) => {
              let name = self.allocator.alloc_str(&string_literal);
              self.next_token()?;
              required_token!(self, Token::Colon);

              let property = ObjectBindingProperty {
                property: PropertyName::Static(name),
                binding: self.read_binding_pattern_with_initializer()?,
              };

              properties.push(property);
            }
            Token::NumberLiteral(number_literal) => {
              let name = self
                .allocator
                .alloc_str(&es_number_to_string(*number_literal, 10));

              self.next_token()?;
              required_token!(self, Token::Colon);

              let property = ObjectBindingProperty {
                property: PropertyName::Static(name),
                binding: self.read_binding_pattern_with_initializer()?,
              };

              properties.push(property);
            }
            Token::BigIntLiteral(_) => {
              todo!()
            }
            Token::TripleStop => {
              self.next_token()?;
              break Some(BindingPattern::Identifier(self.read_binding_identifier()?));
            }
            _ => return Err(syntax_err!()),
          }

          match &self.context.token {
            Token::Comma => self.next_token()?,
            Token::RightCurlyBracket => break None,
            _ => return Err(syntax_err!()),
          }
        };

        required_token!(self, Token::RightCurlyBracket);
        BindingPattern::Object(
          self
            .allocator
            .alloc(ObjectBindingPattern { properties, rest }),
        )
      }
      Token::Name(_) => BindingPattern::Identifier(self.read_binding_identifier()?),
      _ => return Err(syntax_err!()),
    };

    Ok(pattern)
  }

  pub fn read_binding_identifier(&mut self) -> Result<&'a str, ParseError> {
    let pattern = match &self.context.token {
      Token::Name(name) => match self.name_as_binding_identifier(name)? {
        Some(string) => {
          self.next_token()?;
          string
        }
        _ => return Err(syntax_err!()),
      },
      _ => return Err(syntax_err!()),
    };

    Ok(pattern)
  }

  pub fn name_as_binding_identifier(&self, name: &Name) -> Result<Option<&'a str>, ParseError> {
    let string = match name {
      Name::Yield => {
        if self.context.flags.strict_mode || self.context.flags.param_yield {
          return Err(syntax_err!());
        }

        Some("yield")
      }
      Name::Await => {
        if matches!(self.source_type, SourceType::Module) || self.context.flags.param_await {
          return Err(syntax_err!());
        }

        Some("await")
      }
      name => match self.name_as_identifier(name)? {
        None => None,
        Some(string) => {
          if (self.context.flags.strict_mode && matches!(string, "arguments" | "eval"))
            || (self.context.flags.param_await && string == "await")
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
