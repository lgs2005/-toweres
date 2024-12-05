use bumpalo::collections::Vec;

use super::{expression::Expression, object::PropertyName};

#[derive(Debug, Clone, Copy)]
pub enum BindingPattern<'a> {
  Array(&'a ArrayBindingPattern<'a>),
  Identifier(&'a str),
  Object(&'a ObjectBindingPattern<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct BindingPatternInitializer<'a> {
  pub initializer: Option<Expression<'a>>,
  pub pattern: BindingPattern<'a>,
}

#[derive(Debug, Clone)]
pub struct ArrayBindingPattern<'a> {
  pub elements: Vec<'a, Option<BindingPatternInitializer<'a>>>,
  pub rest: Option<BindingPattern<'a>>,
}

#[derive(Debug, Clone)]
pub struct ObjectBindingPattern<'a> {
  pub properties: Vec<'a, ObjectBindingProperty<'a>>,
  pub rest: Option<BindingPattern<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectBindingProperty<'a> {
  pub binding: BindingPatternInitializer<'a>,
  pub property: PropertyName<'a>,
}
