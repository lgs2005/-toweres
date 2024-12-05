use bumpalo::collections::Vec;
use expression::Expression;
use statement::Statement;

pub mod binding;
pub mod class;
pub mod expression;
pub mod function;
pub mod object;
pub mod op;
pub mod statement;

#[derive(Debug, Clone, Copy)]
pub enum SourceType {
  Script,
  Module,
}

#[derive(Debug, Clone)]
pub struct Program<'a> {
  pub source_type: SourceType,
  pub statement_list: Vec<'a, Statement<'a>>,
}
