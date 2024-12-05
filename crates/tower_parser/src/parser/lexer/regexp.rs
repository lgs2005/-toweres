use crate::parser::{
  error::{syntax_err, ParseError, ParseErrorCode},
  Parser,
};

use super::{identifier_utils::is_id_continue, token::Token};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_regexp_literal(&mut self) -> Result<Token, ParseError> {
    let mut source = String::new();

    loop {
      match self.required_char(self.context.position)? {
        '/' => {
          self.context.position += 1;
          break;
        }
        '\\' => {
          source.push('\\');
          match self.required_char(self.context.position + 1)? {
            '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => {
              return Err(syntax_err!());
            }
            c => source.push(*c),
          }
          self.context.position += 2;
        }
        '[' => loop {
          match self.required_char(self.context.position)? {
            ']' => {
              source.push(']');
              self.context.position += 1;
              break;
            }
            '\\' => {
              source.push('\\');
              match self.required_char(self.context.position + 1)? {
                '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => {
                  return Err(syntax_err!());
                }
                c => source.push(*c),
              }
              self.context.position += 2;
            }
            '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => {
              return Err(syntax_err!());
            }
            c => {
              source.push(*c);
              self.context.position += 1;
            }
          }
        },
        '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => {
          return Err(syntax_err!());
        }
        c => {
          source.push(*c);
          self.context.position += 1;
        }
      }
    }

    let mut flags = String::new();
    loop {
      match self.source.get(self.context.position) {
        Some(c) if is_id_continue(*c) => {
          flags.push(*c);
        }
        _ => break,
      }
    }

    Ok(Token::RegExp(source, flags))
  }
}
