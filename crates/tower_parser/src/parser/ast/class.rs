use bumpalo::collections::Vec;

use super::{
  binding::BindingPatternInitializer, expression::Expression, function::FormalParameters,
  statement::Statement,
};

#[derive(Debug, Clone)]
pub struct ClassDefinition<'a> {
  pub identifier: Option<&'a str>,
  pub heritage: Option<Expression<'a>>,
  pub body: Vec<'a, ClassElement<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ClassElement<'a> {
  Field(&'a ClassField<'a>),
  Getter(&'a ClassGetter<'a>),
  Method(&'a ClassMethod<'a>),
  Setter(&'a ClassSetter<'a>),
  Static(&'a Vec<'a, Statement<'a>>),
}

#[derive(Debug, Clone, Copy)]
pub enum ClassElementName<'a> {
  Computed(Expression<'a>),
  Private(&'a str),
  Static(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub struct ClassField<'a> {
  pub name: ClassElementName<'a>,
  pub r#static: bool,
  pub value: Option<Expression<'a>>,
}

#[derive(Debug, Clone)]
pub struct ClassMethod<'a> {
  pub r#async: bool,
  pub body: Vec<'a, Statement<'a>>,
  pub generator: bool,
  pub name: ClassElementName<'a>,
  pub parameters: FormalParameters<'a>,
  pub r#static: bool,
}

#[derive(Debug, Clone)]
pub struct ClassGetter<'a> {
  pub body: Vec<'a, Statement<'a>>,
  pub name: ClassElementName<'a>,
  pub r#static: bool,
}

#[derive(Debug, Clone)]
pub struct ClassSetter<'a> {
  pub body: Vec<'a, Statement<'a>>,
  pub name: ClassElementName<'a>,
  pub parameter: BindingPatternInitializer<'a>,
  pub r#static: bool,
}
