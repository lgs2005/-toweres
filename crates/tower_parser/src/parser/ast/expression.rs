use bumpalo::collections::Vec;

use crate::bigint::BigInt;

use super::{
  class::ClassDefinition,
  function::{Argument, ArrowFunctionDefinition, FunctionDefinition},
  object::PropertyDefinition,
  op::{AssignmentOpExpression, BinaryOpExpression, UnaryOpExpression},
};

#[derive(Debug, Clone, Copy)]
pub enum Expression<'a> {
  Array(&'a Vec<'a, ArrayElement<'a>>),
  ArrowFunction(&'a ArrowFunctionDefinition<'a>),
  Assignment(&'a AssignmentOpExpression<'a>),
  BigInt(&'a BigInt),
  BinaryOp(&'a BinaryOpExpression<'a>),
  Boolean(bool),
  Call(&'a CallExpression<'a>),
  Class(&'a ClassDefinition<'a>),
  Conditional(&'a ConditionalExpression<'a>),
  Group(&'a Expression<'a>),
  Function(&'a FunctionDefinition<'a>),
  Identifier(&'a str),
  Import(&'a Expression<'a>),
  ImportMeta,
  In(&'a InExpression<'a>),
  List(&'a Vec<'a, Expression<'a>>),
  Member(&'a MemberExpression<'a>),
  NewTarget,
  New(&'a NewExpression<'a>),
  Null,
  Number(&'a f64),
  Object(&'a Vec<'a, PropertyDefinition<'a>>),
  RegExp(&'a RegExpLiteral<'a>),
  String(&'a str),
  Super,
  TaggedTemplate(&'a TaggedTemplateLiteral<'a>),
  Template(&'a TemplateLiteral<'a>),
  This,
  UnaryOp(&'a UnaryOpExpression<'a>),
  Yield(&'a YieldExpression<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum ArrayElement<'a> {
  Elision,
  Expression(Expression<'a>),
  Spread(Expression<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct RegExpLiteral<'a> {
  pub flags: &'a str,
  pub source: &'a str,
}

#[derive(Debug, Clone)]
pub struct TemplateLiteral<'a> {
  pub strings: Vec<'a, &'a str>,
  pub substitutions: Vec<'a, Expression<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub enum MemberName<'a> {
  Computed(Expression<'a>),
  Private(&'a str),
  Static(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub struct MemberExpression<'a> {
  pub object: Expression<'a>,
  pub optional: bool,
  pub property: MemberName<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct InExpression<'a> {
  pub name: MemberName<'a>,
  pub argument: Expression<'a>,
}

#[derive(Debug, Clone)]
pub struct TaggedTemplateLiteral<'a> {
  pub optional: bool,
  pub raw_strings: Vec<'a, &'a str>,
  pub strings: Vec<'a, Option<&'a str>>,
  pub substitutions: Vec<'a, Expression<'a>>,
  pub tag: Expression<'a>,
}

#[derive(Debug, Clone)]
pub struct NewExpression<'a> {
  pub arguments: Option<Vec<'a, Argument<'a>>>,
  pub callee: Expression<'a>,
}

#[derive(Debug, Clone)]
pub struct CallExpression<'a> {
  pub arguments: Vec<'a, Argument<'a>>,
  pub callee: Expression<'a>,
  pub optional: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ConditionalExpression<'a> {
  pub alternate: Expression<'a>,
  pub condition: Expression<'a>,
  pub consequent: Expression<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum YieldExpression<'a> {
  All(Expression<'a>),
  Argument(Expression<'a>),
  Empty,
}
