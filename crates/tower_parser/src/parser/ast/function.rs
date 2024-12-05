use bumpalo::collections::Vec;

use super::{
  binding::{BindingPattern, BindingPatternInitializer},
  expression::Expression,
  statement::Statement,
};

#[derive(Debug, Clone)]
pub struct FormalParameters<'a> {
  pub bindings: Vec<'a, BindingPatternInitializer<'a>>,
  pub rest: Option<BindingPattern<'a>>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition<'a> {
  pub r#async: bool,
  pub body: Vec<'a, Statement<'a>>,
  pub generator: bool,
  pub identifier: Option<&'a str>,
  pub parameters: FormalParameters<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum Argument<'a> {
  Positional(Expression<'a>),
  Spread(Expression<'a>),
}

#[derive(Debug, Clone)]
pub struct ArrowFunctionDefinition<'a> {
  pub r#async: bool,
  pub body: Vec<'a, Statement<'a>>,
  pub parameters: FormalParameters<'a>,
}
