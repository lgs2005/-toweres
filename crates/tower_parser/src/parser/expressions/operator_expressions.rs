use crate::parser::{
  ast::{
    expression::{ConditionalExpression, Expression, InExpression, MemberName},
    op::{BinaryOp, BinaryOpExpression, UnaryOp, UnaryOpExpression},
  },
  error::{ParseError, ParseErrorCode},
  lexer::token::{Name, Token},
  required_token, syntax_err, Parser,
};

macro_rules! parse_binary_op_expression {
  ($self:ident.$method:ident, $($a:path => $b:path),*) => {
    match $self.$method()? {
      Some(mut expression) => {
        loop {
          let op = match &$self.context.token {
            $($a => $b),*,
            _ => break,
          };

          $self.next_token()?;
          let argument = $self.$method()?.ok_or(syntax_err!())?;

          let new_expr = BinaryOpExpression {
            left: expression,
            op,
            right: argument,
          };

          expression = Expression::BinaryOp($self.allocator.alloc(new_expr));
        }

        Ok(Some(expression))
      },
      None => Ok(None),
    }
  }
}

macro_rules! parse_single_binary_op_expression {
  ($self:ident.$method:ident, $a:path => $b:path) => {
    match $self.$method()? {
      Some(mut expression) => {
        while matches!($self.context.token, $a) {
          $self.next_token()?;
          let argument = $self.$method()?.ok_or(syntax_err!())?;

          let new_expr = BinaryOpExpression {
            left: expression,
            op: $b,
            right: argument,
          };

          expression = Expression::BinaryOp($self.allocator.alloc(new_expr));
        }

        Ok(Some(expression))
      }
      None => Ok(None),
    }
  };
}

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_conditional_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    match self.read_short_circuit_expression()? {
      Some(condition) => match &self.context.token {
        Token::QuestionMark => {
          self.next_token()?;
          let consequent = self.read_assignment_expression()?.ok_or(syntax_err!())?;
          required_token!(self, Token::Colon);
          let alternate = self.read_assignment_expression()?.ok_or(syntax_err!())?;
          let expression = ConditionalExpression {
            alternate,
            condition,
            consequent,
          };

          Ok(Some(Expression::Conditional(
            self.allocator.alloc(expression),
          )))
        }
        _ => Ok(Some(condition)),
      },
      None => Ok(None),
    }
  }

  fn read_short_circuit_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    match self.read_coalesce_expression()? {
      expr @ Some(_) => Ok(expr),
      None => self.read_logical_or_expression(),
    }
  }

  fn read_coalesce_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    let snapshot = self.context.clone();

    match self.read_bitwise_or_expression()? {
      Some(mut expression) => {
        if !matches!(self.context.token, Token::DoubleQuestionMark) {
          self.context = snapshot;
          return Ok(None);
        }

        while matches!(self.context.token, Token::DoubleQuestionMark) {
          self.next_token()?;
          let argument = self.read_bitwise_or_expression()?.ok_or(syntax_err!())?;

          let new_expr = BinaryOpExpression {
            left: expression,
            op: BinaryOp::Coalesce,
            right: argument,
          };

          expression = Expression::BinaryOp(self.allocator.alloc(new_expr));
        }

        Ok(Some(expression))
      }
      None => Ok(None),
    }
  }

  fn read_logical_or_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    parse_single_binary_op_expression!(
      self.read_logical_and_expression,
      Token::DoubleVerticalLine => BinaryOp::LogicalOr
    )
  }

  fn read_logical_and_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    parse_single_binary_op_expression!(
      self.read_bitwise_or_expression,
      Token::DoubleAmpersand => BinaryOp::LogicalAnd
    )
  }

  fn read_bitwise_or_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    parse_single_binary_op_expression!(
      self.read_bitwise_xor_expression,
      Token::VerticalLine => BinaryOp::BitwiseOr
    )
  }

  fn read_bitwise_xor_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    parse_single_binary_op_expression!(
      self.read_bitwise_and_expression,
      Token::Circumflex => BinaryOp::BitwiseXor
    )
  }

  fn read_bitwise_and_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    parse_single_binary_op_expression!(
      self.read_equality_expression,
      Token::Ampersand => BinaryOp::BitwiseAnd
    )
  }

  fn read_equality_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    parse_binary_op_expression!(
      self.read_relational_expression,
      Token::DoubleEquals => BinaryOp::Equality,
      Token::ExclamationEquals => BinaryOp::Inequality,
      Token::TripleEquals => BinaryOp::StrictEquality,
      Token::ExclamationDoubleEquals => BinaryOp::StrictInequality
    )
  }

  fn read_relational_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    let mut expression = match self.read_shift_expression()? {
      Some(expr) => expr,
      None => match &self.context.token {
        Token::NumberSign => {
          self.next_token()?;

          let name = match &self.context.token {
            Token::Name(name) => self.allocator.alloc_str(name.as_string()),
            _ => return Err(syntax_err!()),
          };

          self.next_token()?;
          required_token!(self, Token::Name(Name::In));

          let argument = self.read_shift_expression()?.ok_or(syntax_err!())?;
          let expression = InExpression {
            argument,
            name: MemberName::Private(name),
          };

          Expression::In(self.allocator.alloc(expression))
        }
        _ => return Ok(None),
      },
    };

    loop {
      let op = match &self.context.token {
        Token::LessThan => BinaryOp::LessThan,
        Token::GreaterThan => BinaryOp::GreaterThan,
        Token::LessThanEquals => BinaryOp::LessThanOrEqual,
        Token::GreaterThanEquals => BinaryOp::GreaterThanOrEqual,
        Token::Name(Name::Instanceof) => BinaryOp::Instanceof,
        Token::Name(Name::In) if self.context.flags.param_in => {
          self.next_token()?;
          let argument = self.read_shift_expression()?.ok_or(syntax_err!())?;
          let new_expr = InExpression {
            argument,
            name: MemberName::Computed(expression),
          };

          expression = Expression::In(self.allocator.alloc(new_expr));
          continue;
        }
        _ => break,
      };

      self.next_token()?;
      let argument = self.read_shift_expression()?.ok_or(syntax_err!())?;

      let new_expr = BinaryOpExpression {
        left: expression,
        op,
        right: argument,
      };

      expression = Expression::BinaryOp(self.allocator.alloc(new_expr));
    }

    Ok(Some(expression))
  }

  fn read_shift_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    parse_binary_op_expression!(
      self.read_additive_expression,
      Token::DoubleLessThan => BinaryOp::LeftShift,
      Token::DoubleGreaterThan => BinaryOp::RightShift,
      Token::TripleGreaterThan => BinaryOp::UnsignedRightShift
    )
  }

  fn read_additive_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    parse_binary_op_expression!(
      self.read_multiplicative_expression,
      Token::Plus => BinaryOp::Addition,
      Token::Minus => BinaryOp::Subtraction
    )
  }

  fn read_multiplicative_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    parse_binary_op_expression!(
      self.read_exponentiation_expression,
      Token::Asterisk => BinaryOp::Multiplication,
      Token::Solidus => BinaryOp::Division,
      Token::Percent => BinaryOp::Remainder
    )
  }

  fn read_exponentiation_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    match self.read_update_expression()? {
      Some(left) => match &self.context.token {
        Token::DoubleAsterisk => {
          self.next_token()?;
          let right = self
            .read_exponentiation_expression()?
            .ok_or(syntax_err!())?;

          let expression = BinaryOpExpression {
            left,
            op: BinaryOp::Exponentiation,
            right,
          };

          Ok(Some(Expression::BinaryOp(self.allocator.alloc(expression))))
        }
        _ => Ok(Some(left)),
      },
      _ => self.read_unary_expression(),
    }
  }

  fn read_unary_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    let op = match &self.context.token {
      Token::Name(Name::Delete) => UnaryOp::Delete,
      Token::Name(Name::Void) => UnaryOp::Void,
      Token::Name(Name::Typeof) => UnaryOp::Typeof,
      Token::Plus => UnaryOp::Absolute,
      Token::Minus => UnaryOp::Negate,
      Token::Tilde => UnaryOp::BitwiseNot,
      Token::Exclamation => UnaryOp::LogicalNot,
      Token::Name(Name::Await) if self.context.flags.param_await => UnaryOp::Await,
      _ => return self.read_update_expression(),
    };

    self.next_token()?;
    let argument = self.read_unary_expression()?.ok_or(syntax_err!())?;
    let expression = self.allocator.alloc(UnaryOpExpression { argument, op });

    Ok(Some(Expression::UnaryOp(expression)))
  }

  fn read_update_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    let expression = match &self.context.token {
      Token::DoublePlus => {
        self.next_token()?;
        let argument = self.read_unary_expression()?.ok_or(syntax_err!())?;
        let expression = UnaryOpExpression {
          argument,
          op: UnaryOp::PrefixIncrement,
        };

        Expression::UnaryOp(self.allocator.alloc(expression))
      }
      Token::DoubleMinus => {
        self.next_token()?;
        let argument = self.read_unary_expression()?.ok_or(syntax_err!())?;
        let expression = UnaryOpExpression {
          argument,
          op: UnaryOp::PrefixDecrement,
        };

        Expression::UnaryOp(self.allocator.alloc(expression))
      }
      _ => match self.read_left_hand_side_expression()? {
        Some(argument) => {
          if self.context.line_terminator {
            argument
          } else {
            match &self.context.token {
              Token::DoublePlus => {
                self.next_token()?;
                let expression = UnaryOpExpression {
                  argument,
                  op: UnaryOp::PostfixIncrement,
                };

                Expression::UnaryOp(self.allocator.alloc(expression))
              }
              Token::DoubleMinus => {
                self.next_token()?;
                let expression = UnaryOpExpression {
                  argument,
                  op: UnaryOp::PostfixDecrement,
                };

                Expression::UnaryOp(self.allocator.alloc(expression))
              }
              _ => argument,
            }
          }
        }
        None => return Ok(None),
      },
    };

    Ok(Some(expression))
  }
}
