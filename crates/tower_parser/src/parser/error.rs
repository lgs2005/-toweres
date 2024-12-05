use std::{
  error::Error,
  fmt::{Debug, Display},
};

#[derive(Debug, Clone, Copy)]
pub enum ParseErrorCode {
  InvalidEscape,
  InvalidTemplateString,
  InvalidUnicode,
  StrictOctalLiteral,
  StrictOctalEscape,
  SyntaxError,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseError {
  pub code: ParseErrorCode,
  pub module_path: &'static str,
  pub line: u32,
}

impl Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Debug::fmt(self, f)
  }
}

impl Error for ParseError {}

macro_rules! parse_err {
  ($code:path) => {
    ParseError {
      code: $code,
      module_path: module_path!(),
      line: line!(),
    }
  };
}

macro_rules! syntax_err {
  () => {
    ParseError {
      code: ParseErrorCode::SyntaxError,
      module_path: module_path!(),
      line: line!(),
    }
  };
}

macro_rules! required_token {
  ($self:ident, $token:pat) => {
    if matches!($self.context.token, $token) {
      $self.next_token()?;
    } else {
      Err(ParseError {
        code: ParseErrorCode::SyntaxError,
        module_path: module_path!(),
        line: line!(),
      })?;
    }
  };
}

pub(crate) use parse_err;
pub(crate) use required_token;
pub(crate) use syntax_err;
