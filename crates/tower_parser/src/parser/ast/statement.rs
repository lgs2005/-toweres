use bumpalo::collections::Vec;

use super::{
  binding::{BindingPattern, BindingPatternInitializer},
  expression::Expression,
};

#[derive(Debug, Clone, Copy)]
pub enum Statement<'a> {
  Block(&'a Vec<'a, Statement<'a>>),
  Break(&'a Option<&'a str>),
  Continue(&'a Option<&'a str>),
  Debugger,
  DoWhile(&'a DoWhileStatement<'a>),
  Expression(&'a Expression<'a>),
  Empty,
  If(&'a IfStatement<'a>),
  Label(&'a LabelStatement<'a>),
  Return(&'a Option<Expression<'a>>),
  Switch(&'a SwitchStatement<'a>),
  Throw(&'a Expression<'a>),
  Try(&'a TryStatement<'a>),
  Variable(&'a Vec<'a, BindingPatternInitializer<'a>>),
  With(&'a WithStatement<'a>),
  While(&'a WhileStatement<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct IfStatement<'a> {
  pub alternate: Option<Statement<'a>>,
  pub condition: Expression<'a>,
  pub consequent: Statement<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct DoWhileStatement<'a> {
  pub condition: Expression<'a>,
  pub body: Statement<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct WhileStatement<'a> {
  pub condition: Expression<'a>,
  pub body: Statement<'a>,
}

#[derive(Debug, Clone)]
pub struct SwitchStatement<'a> {
  pub cases: Vec<'a, SwitchCase<'a>>,
  pub expression: Expression<'a>,
}

#[derive(Debug, Clone)]
pub struct SwitchCase<'a> {
  pub expression: Option<Expression<'a>>,
  pub body: Vec<'a, Statement<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct WithStatement<'a> {
  pub expression: Expression<'a>,
  pub body: Statement<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct LabelStatement<'a> {
  pub label: &'a str,
  pub statement: Statement<'a>,
}

#[derive(Debug, Clone)]
pub struct TryStatement<'a> {
  pub body: Vec<'a, Statement<'a>>,
  pub catch: Option<CatchBlock<'a>>,
  pub finally: Option<Vec<'a, Statement<'a>>>,
}

#[derive(Debug, Clone)]
pub struct CatchBlock<'a> {
  pub parameter: Option<BindingPattern<'a>>,
  pub body: Vec<'a, Statement<'a>>,
}
