use bumpalo::collections::Vec;

use super::{
  ast::{
    expression::{Expression, YieldExpression},
    op::{AssignmentOp, AssignmentOpExpression},
  },
  error::{ParseError, ParseErrorCode},
  lexer::token::{Name, Token},
  syntax_err, Parser,
};

mod class;
mod function;
mod identifier;
mod left_hand_side_expression;
mod object_literal;
mod operator_expressions;
mod primary_expression;
mod template;

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    match self.read_assignment_expression()? {
      Some(expr) => match &self.context.token {
        Token::Comma => {
          let list = self.read_expression_list(expr)?;
          Ok(Some(Expression::List(self.allocator.alloc(list))))
        }
        _ => Ok(Some(expr)),
      },
      None => Ok(None),
    }
  }

  fn read_expression_list(
    &mut self,
    init: Expression<'a>,
  ) -> Result<Vec<'a, Expression<'a>>, ParseError> {
    let mut list = Vec::<Expression<'a>>::new_in(&self.allocator);
    list.push(init);

    loop {
      match &self.context.token {
        Token::Comma => {
          self.next_token()?;
          let expression = self.read_assignment_expression()?.ok_or(syntax_err!())?;
          list.push(expression);
        }
        _ => break,
      }
    }

    Ok(list)
  }

  pub fn read_assignment_expression(&mut self) -> Result<Option<Expression<'a>>, ParseError> {
    let higher_expr = match &self.context.token {
      Token::LeftParenthesis => self.read_arrow_function_expression(false)?,
      Token::Name(Name::Async) => {
        let snapshot = self.context.clone();
        self.next_token()?;
        if self.context.line_terminator {
          return Err(syntax_err!());
        }

        match self.read_arrow_function_expression(true)? {
          Some(expr) => Some(expr),
          None => {
            self.context = snapshot;
            None
          }
        }
      }
      Token::Name(Name::Yield) if self.context.flags.param_yield => {
        self.next_token()?;

        let expr = if self.context.line_terminator {
          Expression::Yield(&YieldExpression::Empty)
        } else {
          match &self.context.token {
            Token::Asterisk => {
              self.next_token()?;
              let argument = self.read_assignment_expression()?.ok_or(syntax_err!())?;
              let expression = self.allocator.alloc(YieldExpression::All(argument));
              Expression::Yield(expression)
            }
            _ => match self.read_assignment_expression()? {
              Some(expr) => {
                let expression = self.allocator.alloc(YieldExpression::Argument(expr));
                Expression::Yield(expression)
              }
              None => Expression::Yield(&YieldExpression::Empty),
            },
          }
        };

        Some(expr)
      }
      _ => None,
    };

    if let Some(expr) = higher_expr {
      return Ok(Some(expr));
    }

    let snapshot = self.context.clone();

    match self.read_conditional_expression()? {
      Some(expr) => match &self.context.token {
        Token::Equals
        | Token::AsteriskEquals
        | Token::SolidusEquals
        | Token::PlusEquals
        | Token::MinusEquals
        | Token::DoubleLessThanEquals
        | Token::DoubleGreaterThanEquals
        | Token::TripleGreaterThanEquals
        | Token::AmpersandEquals
        | Token::CircumflexEquals
        | Token::VerticalLineEquals
        | Token::DoubleAsteriskEquals
        | Token::DoubleAmpersandEquals
        | Token::DoubleVerticalLineEquals
        | Token::DoubleQuestionMarkEquals => {
          self.context = snapshot;
        }
        _ => return Ok(Some(expr)),
      },
      _ => self.context = snapshot,
    };

    let left = match self.read_left_hand_side_expression()? {
      Some(expr) => expr,
      None => return Ok(None),
    };

    let op = match &self.context.token {
      Token::Equals => AssignmentOp::Assignment,
      Token::AsteriskEquals => AssignmentOp::Multiplication,
      Token::SolidusEquals => AssignmentOp::Division,
      Token::PlusEquals => AssignmentOp::Addition,
      Token::MinusEquals => AssignmentOp::Subtraction,
      Token::DoubleLessThanEquals => AssignmentOp::LeftShift,
      Token::DoubleGreaterThanEquals => AssignmentOp::RightShift,
      Token::TripleGreaterThanEquals => AssignmentOp::UnsignedRightShift,
      Token::AmpersandEquals => AssignmentOp::BitwiseAnd,
      Token::CircumflexEquals => AssignmentOp::BitwiseXor,
      Token::VerticalLineEquals => AssignmentOp::BitwiseOr,
      Token::DoubleAsteriskEquals => AssignmentOp::Exponentiation,
      Token::DoubleAmpersandEquals => AssignmentOp::LogicalAnd,
      Token::DoubleVerticalLineEquals => AssignmentOp::LogicalOr,
      Token::DoubleQuestionMarkEquals => AssignmentOp::Coalesce,
      _ => unreachable!(),
    };

    let right = self.read_assignment_expression()?.ok_or(syntax_err!())?;
    let expression = AssignmentOpExpression { left, op, right };

    Ok(Some(Expression::Assignment(
      self.allocator.alloc(expression),
    )))
  }
}
