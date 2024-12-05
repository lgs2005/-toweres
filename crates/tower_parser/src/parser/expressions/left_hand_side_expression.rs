use crate::parser::{
  ast::expression::{CallExpression, Expression, MemberExpression, MemberName, NewExpression},
  error::{ParseError, ParseErrorCode},
  lexer::token::{Name, Token},
  required_token, syntax_err, Parser,
};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_left_hand_side_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    // the idea here is to pick an expression to start from, then loop trough calls, optional chains
    // and member accesses until its over
    let mut expression = match &self.context.token {
      Token::Name(Name::Super) => {
        self.next_token()?;
        match &self.context.token {
          // The actual expression will be figured out in the second part of this function
          Token::LeftParenthesis | Token::LeftSquareBracket | Token::FullStop => Expression::Super,
          _ => return Err(syntax_err!()),
        }
      }
      Token::Name(Name::Import) => {
        self.next_token()?;
        match &self.context.token {
          Token::LeftParenthesis => {
            self.next_token()?;
            let argument = self.read_assignment_expression()?.ok_or(syntax_err!())?;
            required_token!(self, Token::RightParenthesis);
            Expression::Import(self.allocator.alloc(argument))
          }
          Token::FullStop => {
            self.next_token()?;
            required_token!(self, Token::Name(Name::Meta));
            Expression::ImportMeta
          }
          _ => return Err(syntax_err!()),
        }
      }
      Token::Name(Name::New) => {
        self.next_token()?;
        match &self.context.token {
          Token::FullStop => {
            self.next_token()?;
            required_token!(self, Token::Name(Name::Target));
            Expression::NewTarget
          }
          // new expressions need special treatment
          // if it has parenthesis, its a MemberExpression production, and the
          // second part of this function is allowed to use it
          // if it doesnt have parenthesis, its a NewExpression production, which
          // is not accepted for CallExpression/OptionalExpression/MemberExpression,
          // so the second part of this function cant use it
          _ => {
            let (expr, is_new_expr) = self.recurse_new_expression()?;
            if is_new_expr {
              return Ok(Some(expr));
            } else {
              expr
            }
          }
        }
      }
      _ => match self.read_primary_expression()? {
        Some(expr) => expr,
        None => return Ok(None),
      },
    };

    loop {
      expression = match &self.context.token {
        Token::QuestionMarkStop => {
          self.next_token()?;
          match &self.context.token {
            Token::LeftParenthesis => {
              let arguments = self.read_arguments()?;
              let expression = CallExpression {
                arguments,
                callee: expression,
                optional: true,
              };
              Expression::Call(self.allocator.alloc(expression))
            }
            _ => match self.read_member_access(expression, true)? {
              Some(expr) => expr,
              None => return Err(syntax_err!()),
            },
          }
        }
        Token::LeftParenthesis => {
          let arguments = self.read_arguments()?;
          let expression = CallExpression {
            arguments,
            callee: expression,
            optional: false,
          };
          Expression::Call(self.allocator.alloc(expression))
        }
        _ => match self.read_member_access(expression, false)? {
          Some(expr) => expr,
          None => break,
        },
      }
    }

    Ok(Some(expression))
  }

  fn recurse_new_expression(&mut self) -> Result<(Expression<'a>, bool), ParseError> {
    let result = match &self.context.token {
      // If the token is `new`, we dont know if its a MemberExpression or NewExpression
      Token::Name(Name::New) => {
        let (expr, is_new_expr) = self.recurse_new_expression()?;
        // if the next expression is a NewExpression, this one is also a NewExpression
        // and cant have parenthesis
        if is_new_expr {
          (expr, true)
        } else {
          match &self.context.token {
            Token::LeftParenthesis => {
              let arguments = self.read_arguments()?;
              let expression = NewExpression {
                arguments: Some(arguments),
                callee: expr,
              };

              // If there are parenthesis, this is a MemberExpression
              (Expression::New(self.allocator.alloc(expression)), false)
            }
            _ => {
              let expression = NewExpression {
                arguments: None,
                callee: expr,
              };

              // if there arent parenthesis, this is a NewExpression
              (Expression::New(self.allocator.alloc(expression)), true)
            }
          }
        }
      }
      // but otherwise, its a member expression
      // this works like `read_left_hand_side_expression`, but only matches the MemberExpression production
      _ => {
        let mut expression = match &self.context.token {
          Token::Name(Name::Import) => {
            self.next_token()?;
            required_token!(self, Token::FullStop);
            required_token!(self, Token::Name(Name::Meta));
            Expression::ImportMeta
          }
          Token::Name(Name::Super) => {
            self.next_token()?;
            match &self.context.token {
              Token::LeftSquareBracket | Token::FullStop => Expression::Super,
              _ => return Err(syntax_err!()),
            }
          }
          _ => self.read_primary_expression()?.ok_or(syntax_err!())?,
        };

        while let Some(expr) = self.read_member_access(expression, false)? {
          expression = expr;
        }

        (expression, false)
      }
    };

    Ok(result)
  }

  fn read_member_access(
    &mut self,
    object: Expression<'a>,
    optional: bool,
  ) -> Result<Option<Expression<'a>>, ParseError> {
    let expression = match &self.context.token {
      Token::LeftSquareBracket => {
        self.next_token()?;
        let property_expr = self.read_expression()?.ok_or(syntax_err!())?;
        let property = MemberName::Computed(property_expr);
        required_token!(self, Token::RightSquareBracket);
        let expression = MemberExpression {
          object,
          optional,
          property,
        };
        Expression::Member(self.allocator.alloc(expression))
      }
      Token::FullStop => {
        self.next_token()?;

        let property = match &self.context.token {
          Token::NumberSign => {
            self.next_token()?;
            let identifier = match &self.context.token {
              Token::Name(name) => self.allocator.alloc_str(name.as_string()),
              _ => return Err(syntax_err!()),
            };
            self.next_token()?;
            MemberName::Private(identifier)
          }
          Token::Name(name) => {
            let identifier = self.allocator.alloc_str(name.as_string());
            self.next_token()?;
            MemberName::Static(identifier)
          }
          _ => return Err(syntax_err!()),
        };

        let expression = MemberExpression {
          object,
          optional,
          property,
        };
        Expression::Member(self.allocator.alloc(expression))
      }
      Token::TemplateHead(_, _) => self.read_tagged_template_literal(object, optional)?,
      _ => return Ok(None),
    };

    Ok(Some(expression))
  }
}

/*
 * SEE ALSO: some pseudocode-ish thing
MemberExpression:
  PrimaryExpression
  MemberExpression [ Expression ]
  MemberExpression . IndentifierName
  MemberExpression TemplateLiteral
  super [ Expression ]
  super . IndentifierName
  new . target
  import . meta
  new MemberExpression Arguments
  MemberExpression . PrivateIdentifier
NewExpression:
  MemberExpression
  new NewExpression
CallExpression:
  MemberExpression Arguments
  super Arguments
  import ( AssignmentExpression )
  CallExpression Arguments
  CallExpression [ Expression ]
  CallExpression . IdentifierName
  CallExpression TemplateLiteral
  CallExpression . PrivateIndentifier
OptionalExpression:
  MemberExpression OptionalChain
  CallExpression OptionalChain
  OptionalExpression OptionalChain
OptionalChain:
  ?. Arguments
  ?. [ Expression ]
  ?. IdentifierName
  ?. TemplateLiteral
  ?. PrivateIdentifier
  OptionalChain Arguments
  OptionalChain [ Expression ]
  OptionalChain . IdentifierName
  OptionalChain TemplateLiteral
  OptionalChain . PrivateIdentifier
LeftHandSideExpression:
  NewExpression
  CallExpression
  OptionalExpression

read_member_expression -> Expression:
let expression = match token {
  import => {
    read (import . meta)
  }
  super => {
    match next prod {
      super [ Expression ]
      super . IdentifierName
    }
  }
  new => {
    read (new MemberExpression Arguments)
  }
  _ => read_primary_expression(),
}

loop {
  match token {
    [ => append [ Expression ]
    TemplateHead => append TemplateLiteral
    . => {
      # => append . PrivateIdentifier
      _ => append . IdentifierName
    }
  }
}


read_new_expression -> (Expression, bool):
skip_token(new)
if next token is new {
  (expr, is_new_expr) = read_new_expression()
  if is_new_expr {
    return (expr, true) (as NewExpression)
  } else {
    if next token is LeftParenthesis {
      arguments = read_arguments()
      return (NewExpression { callee: expr, arguments: Some(arguments) }, false) (as MemberExpression)
    } else {
      return (NewExpression { callee: expr, arguments: None }, true) (as NewExpression)
    }
  }
} else {
  return (read_member_expression(), false) (as MemberExpression)
}

read_left_hand_side_expression -> Expression:
let expression = match token {
  super => match next prod {
    super Arguments
    super [ Expression ]
    super . IdentifierName
  }
  import => match next prod {
    import ( AssignmentExpression )
    import . meta
  }
  new => {
    if matches new . target {
      new . target
    } else {
      (expr, is_new_expr) = read_new_expression()
      if is_new_expr {
        return expr (as NewExpression)
      } else {
        expr
      }
    }
  }
  _ => read_primary_expression(),
}

loop {
  match token {
    ( => append Arguments,
    [ => append [ Expression ]
    TemplateHead => append TemplateLiteral
    . => {
      # => append . PrivateIdentifier
      _ => append . IdentifierName
    }
    ?. => require one of previous, append
  }
}
 */
