use ast::{statement::Statement, Program, SourceType};
use bumpalo::{collections::Vec, Bump};
use error::{parse_err, required_token, syntax_err, ParseError, ParseErrorCode};
use lexer::token::Token;

pub mod ast;
mod binding;
mod error;
mod expressions;
mod lexer;
mod statements;

pub fn parse_source<'a>(
  allocator: &'a Bump,
  source: &'a [char],
  source_type: SourceType,
) -> Result<&'a Program<'a>, ParseError> {
  Parser::new(&allocator, &source, source_type).parse_source()
}

pub struct Parser<'r, 'a: 'r> {
  source: &'r [char],
  allocator: &'a Bump,
  source_type: SourceType,
  context: ParsingContext,
}

#[derive(Debug, Clone)]
pub struct ParsingContext {
  pub position: usize,
  pub token: Token,
  pub line_terminator: bool,
  pub flags: ParserFlags,
}

#[derive(Debug, Clone, Copy)]
pub struct ParserFlags {
  pub strict_mode: bool,
  pub goal_regexp: bool,
  pub goal_template: bool,
  pub param_await: bool,
  pub param_yield: bool,
  pub param_in: bool,
}

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn new(allocator: &'a Bump, source: &'a [char], source_type: SourceType) -> Self {
    Self {
      allocator,
      source_type,
      source,
      context: ParsingContext {
        position: 0,
        token: Token::EndOfInput,
        line_terminator: false,
        flags: ParserFlags {
          strict_mode: matches!(source_type, SourceType::Module),
          goal_regexp: true,
          goal_template: false,
          param_await: matches!(source_type, SourceType::Module),
          param_yield: false,
          param_in: false,
        },
      },
    }
  }

  pub fn parse_source(&mut self) -> Result<&'a Program<'a>, ParseError> {
    self.next_token()?;
    let mut list = Vec::<Statement<'a>>::new_in(self.allocator);

    loop {
      match &self.context.token {
        Token::EndOfInput => break,
        _ => {
          let statement = self.read_statement()?.ok_or(syntax_err!())?;
          list.push(statement);
        }
      }
    }

    let program = Program {
      source_type: self.source_type,
      statement_list: list,
    };

    Ok(self.allocator.alloc(program))
  }

  pub fn auto_semicolon(&mut self) -> Result<(), ParseError> {
    match &self.context.token {
      Token::Semicolon => self.next_token(),
      Token::RightCurlyBracket | Token::EndOfInput if self.context.line_terminator => Ok(()),
      _ => Err(syntax_err!()),
    }
  }
}
