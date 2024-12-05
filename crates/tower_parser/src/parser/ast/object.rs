use bumpalo::collections::Vec;

use super::{
  binding::BindingPatternInitializer, function::FormalParameters, Expression, Statement,
};

#[derive(Debug, Clone, Copy)]
pub enum PropertyName<'a> {
  Computed(Expression<'a>),
  Static(&'a str),
}

#[derive(Debug, Clone)]
pub struct ObjectMethod<'a> {
  pub r#async: bool,
  pub body: Vec<'a, Statement<'a>>,
  pub generator: bool,
  pub parameters: FormalParameters<'a>,
  pub property: PropertyName<'a>,
}

#[derive(Debug, Clone)]
pub struct ObjectGetter<'a> {
  pub body: Vec<'a, Statement<'a>>,
  pub property: PropertyName<'a>,
}

#[derive(Debug, Clone)]
pub struct ObjectSetter<'a> {
  pub body: Vec<'a, Statement<'a>>,
  pub parameter: BindingPatternInitializer<'a>,
  pub property: PropertyName<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct PropertyDefinition<'a> {
  pub expression: Expression<'a>,
  pub property: PropertyName<'a>,
}

#[derive(Debug, Clone)]
pub enum ObjectProperty<'a> {
  Getter(&'a ObjectGetter<'a>),
  Method(&'a ObjectMethod<'a>),
  Property(&'a PropertyDefinition<'a>),
  Setter(&'a ObjectSetter<'a>),
  Shorthand(&'a str),
  Spread(Expression<'a>),
}
