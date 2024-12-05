use crate::parser::{
  ast::{expression::Expression, SourceType},
  error::{ParseError, ParseErrorCode},
  lexer::token::{Name, Token},
  syntax_err, Parser,
};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_identifier_reference(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    match &self.context.token {
      Token::Name(name) => match self.name_as_identifier_reference(name)? {
        Some(string) => {
          self.next_token()?;
          Ok(Some(Expression::Identifier(string)))
        }
        None => Ok(None),
      },
      _ => Ok(None),
    }
  }

  pub fn name_as_identifier_reference(&self, name: &Name) -> Result<Option<&'a str>, ParseError> {
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

  pub fn name_as_identifier(&self, name: &Name) -> Result<Option<&'a str>, ParseError> {
    let string = match name {
      Name::Async => Some("async"),
      Name::Get => Some("get"),
      Name::Of => Some("of"),
      Name::Meta => Some("meta"),
      Name::Set => Some("set"),
      Name::Target => Some("target"),
      Name::Let => {
        if self.context.flags.strict_mode {
          return Err(syntax_err!());
        }
        Some("let")
      }
      Name::Static => {
        if self.context.flags.strict_mode {
          return Err(syntax_err!());
        }
        Some("static")
      }
      Name::Unclassified(string) => match string.as_str() {
        "break" | "case" | "catch" | "class" | "const" | "continue" | "debugger" | "default"
        | "delete" | "do" | "else" | "enum" | "export" | "extends" | "false" | "finally"
        | "for" | "function" | "if" | "import" | "in" | "instanceof" | "new" | "null"
        | "return" | "super" | "switch" | "this" | "throw" | "true" | "try" | "typeof" | "var"
        | "void" | "while" | "with" => None,
        string => {
          if self.context.flags.strict_mode
            && match string {
              "implements" | "interface" | "let" | "package" | "private" | "protected"
              | "public" | "static" | "yield" => true,
              _ => false,
            }
          {
            return Err(syntax_err!());
          }

          if matches!(self.source_type, SourceType::Module) && string == "await" {
            return Err(syntax_err!());
          }

          let string: &'a str = self.allocator.alloc_str(string);
          Some(string)
        }
      },
      _ => None,
    };

    Ok(string)
  }
}
