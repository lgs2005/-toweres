use crate::parser::{
  error::{syntax_err, ParseError, ParseErrorCode},
  Parser,
};

use super::token::Token;

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_string_literal(&mut self, quote_type: char) -> Result<Token, ParseError> {
    self.context.position += 1;
    let mut characters = String::new();

    loop {
      match self.required_char(self.context.position)? {
        '\u{000A}' | '\u{000D}' => {
          return Err(syntax_err!());
        }
        '\\' => {
          self.context.position += 1;
          match self.read_string_escape_sequence()? {
            None => {}
            Some(c) => characters.push(c),
          }
        }
        c => {
          if *c == quote_type {
            self.context.position += 1;
            break;
          } else {
            characters.push(*c);
            self.context.position += 1;
          }
        }
      }
    }

    Ok(Token::StringLiteral(characters))
  }

  pub fn read_template_characters(&mut self) -> Result<(bool, String, Option<String>), ParseError> {
    let mut characters = String::new();
    let mut has_invalid_character = false;
    let start_index = self.context.position;

    let is_tail = loop {
      match self.required_char(self.context.position)? {
        '`' => {
          self.context.position += 1;
          break true;
        }
        '$' => match self.required_char(self.context.position + 1)? {
          '{' => {
            self.context.position += 2;
            break false;
          }
          _ => {
            self.context.position += 1;
            characters.push('$')
          }
        },
        '\\' => {
          self.context.position += 1;
          let original_position = self.context.position;
          match self.read_string_escape_sequence() {
            Ok(Some(c)) => characters.push(c),
            Ok(None) => {}
            Err(_) => {
              self.context.position = original_position;
              has_invalid_character = true;
            }
          }
        }
        c => {
          characters.push(*c);
          self.context.position += 1;
        }
      }
    };

    let raw_characters = String::from_iter(&self.source[start_index..(self.context.position - 1)]);

    if has_invalid_character {
      Ok((is_tail, raw_characters, None))
    } else {
      Ok((is_tail, raw_characters, Some(characters)))
    }
  }
}
