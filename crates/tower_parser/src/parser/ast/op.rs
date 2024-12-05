use super::Expression;

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
  Absolute,
  Await,
  BitwiseNot,
  Delete,
  LogicalNot,
  Negate,
  PostfixDecrement,
  PostfixIncrement,
  PrefixDecrement,
  PrefixIncrement,
  Typeof,
  Void, // Wow how strange if you get what i mean hahahahahahaha
}

#[derive(Debug, Clone, Copy)]
pub struct UnaryOpExpression<'a> {
  pub op: UnaryOp,
  pub argument: Expression<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
  Addition,
  BitwiseAnd,
  BitwiseOr,
  BitwiseXor,
  Coalesce,
  Division,
  Equality,
  Exponentiation,
  GreaterThan,
  GreaterThanOrEqual,
  Inequality,
  Instanceof,
  LeftShift,
  LessThan,
  LessThanOrEqual,
  LogicalAnd,
  LogicalOr,
  Multiplication,
  Remainder,
  RightShift,
  StrictEquality,
  StrictInequality,
  Subtraction,
  UnsignedRightShift,
}

#[derive(Debug, Clone, Copy)]
pub struct BinaryOpExpression<'a> {
  pub op: BinaryOp,
  pub left: Expression<'a>,
  pub right: Expression<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum AssignmentOp {
  Addition,
  Assignment,
  BitwiseAnd,
  BitwiseOr,
  BitwiseXor,
  Coalesce,
  Division,
  Exponentiation,
  LeftShift,
  LogicalAnd,
  LogicalOr,
  Multiplication,
  Remainder,
  RightShift,
  Subtraction,
  UnsignedRightShift,
}

#[derive(Debug, Clone, Copy)]
pub struct AssignmentOpExpression<'a> {
  pub op: AssignmentOp,
  pub left: Expression<'a>,
  pub right: Expression<'a>,
}
