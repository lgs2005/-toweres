use std::slice::SliceIndex;

use identifier_utils::{is_id_continue, is_id_start};
use token::{Name, Token};

use crate::parser::{parse_err, syntax_err};

use super::{
  error::{ParseError, ParseErrorCode},
  Parser,
};

mod escape_sequences;
mod identifier_utils;
mod numeric;
mod regexp;
mod strings;

pub mod token;

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn next_token(&mut self) -> Result<(), ParseError> {
    self.context.line_terminator = false;

    loop {
      match self.source.get(self.context.position) {
        None => {
          self.context.token = Token::EndOfInput;
          return Ok(());
        }
        Some(c) => match c {
          '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\u{00020}' | '\u{00A0}'
          | '\u{1680}' | '\u{2000}' | '\u{2001}' | '\u{2002}' | '\u{2003}' | '\u{2004}'
          | '\u{2005}' | '\u{2006}' | '\u{2007}' | '\u{2008}' | '\u{2009}' | '\u{200A}'
          | '\u{202F}' | '\u{205F}' | '\u{3000}' => {
            self.context.position += 1;
          }
          '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => {
            self.context.line_terminator = true;
            self.context.position += 1;
          }
          '/' => match self.source.get(self.context.position + 1) {
            Some('/') => {
              self.context.position += 2;
              loop {
                match self.source.get(self.context.position) {
                  None => break,
                  Some('\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}') => {
                    self.context.line_terminator = true;
                    self.context.position += 1;
                    break;
                  }
                  _ => self.context.position += 1,
                }
              }
            }
            Some('*') => {
              self.context.position += 2;
              loop {
                match self.required_char(self.context.position)? {
                  '*' => match self.required_char(self.context.position + 1)? {
                    '/' => {
                      self.context.position += 2;
                      break;
                    }
                    _ => self.context.position += 1,
                  },
                  '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => {
                    self.context.line_terminator = true;
                    self.context.position += 1;
                  }
                  _ => {
                    self.context.position += 1;
                  }
                }
              }
            }
            _ => break,
          },
          '#' if self.context.position == 0 => match self.source.get(self.context.position + 1) {
            Some('!') => {
              self.context.position += 2;
              loop {
                match self.source.get(self.context.position) {
                  None => break,
                  Some('\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}') => {
                    self.context.line_terminator = true;
                    self.context.position += 1;
                    break;
                  }
                  _ => self.context.position += 1,
                }
              }
            }
            _ => break,
          },
          _ => break,
        },
      }
    }

    macro_rules! simple_punctuator {
      ($token:path) => {{
        self.context.position += 1;
        $token
      }};
    }

    let token = match &self.source[self.context.position] {
      start_char if is_id_start(*start_char) => {
        self.context.position += 1;
        self.read_word_token(*start_char)?
      }
      '\\' => match self.required_char(self.context.position + 1)? {
        'u' => {
          self.context.position += 2;
          let character = self.read_unicode_escape_sequence()?;

          if is_id_start(character) {
            self.read_word_token(character)?
          } else {
            return Err(syntax_err!());
          }
        }
        _ => return Err(syntax_err!()),
      },
      quote_type @ ('"' | '\'') => self.read_string_literal(*quote_type)?,
      '0' => self.read_zero_starting_literal()?,
      digit @ '1'..='9' => self.read_nonzero_starting_literal(*digit)?,
      '.' => match self.source.get(self.context.position + 1) {
        Some('0'..='9') => self.read_decimal_literal(None)?,
        _ => match self.source.get(self.context.position + 1) {
          Some('.') => match self.source.get(self.context.position + 2) {
            Some('.') => {
              self.context.position += 3;
              Token::TripleStop
            }
            _ => {
              self.context.position += 1;
              Token::FullStop
            }
          },
          _ => {
            self.context.position += 1;
            Token::FullStop
          }
        },
      },
      '`' => {
        self.context.position += 1;
        let (reached_end, raw_characters, characters) = self.read_template_characters()?;

        if reached_end {
          Token::NoSubstitutionTemplate(raw_characters, characters)
        } else {
          Token::TemplateHead(raw_characters, characters)
        }
      }
      '}' => {
        self.context.position += 1;
        if !self.context.flags.goal_template {
          Token::RightCurlyBracket
        } else {
          let (reached_end, raw_characters, characters) = self.read_template_characters()?;

          if reached_end {
            Token::TemplateTail(raw_characters, characters)
          } else {
            Token::TemplateMiddle(raw_characters, characters)
          }
        }
      }
      '/' => {
        if !self.context.flags.goal_regexp {
          match self.source.get(self.context.position + 1) {
            Some('=') => {
              self.context.position += 2;
              Token::SolidusEquals
            }
            _ => {
              self.context.position += 1;
              Token::Solidus
            }
          }
        } else {
          self.context.position += 1;
          self.read_regexp_literal()?
        }
      }
      '?' => match self.source.get(self.context.position + 1) {
        Some('.') => match self.source.get(self.context.position + 2) {
          Some('0'..='9') => {
            self.context.position += 1;
            Token::QuestionMark
          }
          _ => {
            self.context.position += 2;
            Token::QuestionMarkStop
          }
        },
        Some('?') => match self.source.get(self.context.position + 2) {
          Some('=') => {
            self.context.position += 3;
            Token::DoubleQuestionMarkEquals
          }
          _ => {
            self.context.position += 2;
            Token::DoubleQuestionMark
          }
        },
        _ => {
          self.context.position += 1;
          Token::QuestionMark
        }
      },
      '<' => match self.source.get(self.context.position + 1) {
        Some('<') => match self.source.get(self.context.position + 2) {
          Some('=') => {
            self.context.position += 3;
            Token::DoubleLessThanEquals
          }
          _ => {
            self.context.position += 2;
            Token::DoubleLessThan
          }
        },
        Some('=') => {
          self.context.position += 2;
          Token::LessThanEquals
        }
        _ => {
          self.context.position += 1;
          Token::LessThan
        }
      },
      '>' => match self.source.get(self.context.position + 1) {
        Some('>') => match self.source.get(self.context.position + 2) {
          Some('>') => match self.source.get(self.context.position + 3) {
            Some('=') => {
              self.context.position += 4;
              Token::TripleGreaterThanEquals
            }
            _ => {
              self.context.position += 3;
              Token::TripleGreaterThan
            }
          },
          Some('=') => {
            self.context.position += 3;
            Token::DoubleGreaterThanEquals
          }
          _ => {
            self.context.position += 2;
            Token::DoubleGreaterThan
          }
        },
        Some('=') => {
          self.context.position += 2;
          Token::GreaterThanEquals
        }
        _ => {
          self.context.position += 1;
          Token::GreaterThan
        }
      },
      '=' => match self.source.get(self.context.position + 1) {
        Some('=') => match self.source.get(self.context.position + 2) {
          Some('=') => {
            self.context.position += 3;
            Token::TripleEquals
          }
          _ => {
            self.context.position += 2;
            Token::DoubleEquals
          }
        },
        Some('>') => {
          self.context.position += 2;
          Token::FatArrow
        }
        _ => {
          self.context.position += 1;
          Token::Equals
        }
      },
      '!' => match self.source.get(self.context.position + 1) {
        Some('=') => match self.source.get(self.context.position + 2) {
          Some('=') => {
            self.context.position += 3;
            Token::ExclamationDoubleEquals
          }
          _ => {
            self.context.position += 2;
            Token::ExclamationEquals
          }
        },
        _ => {
          self.context.position += 1;
          Token::Exclamation
        }
      },
      '+' => match self.source.get(self.context.position + 1) {
        Some('+') => {
          self.context.position += 2;
          Token::DoublePlus
        }
        Some('=') => {
          self.context.position += 2;
          Token::PlusEquals
        }
        _ => {
          self.context.position += 1;
          Token::Plus
        }
      },
      '-' => match self.source.get(self.context.position + 1) {
        Some('-') => {
          self.context.position += 2;
          Token::DoubleMinus
        }
        Some('=') => {
          self.context.position += 2;
          Token::MinusEquals
        }
        _ => {
          self.context.position += 1;
          Token::Minus
        }
      },
      '*' => match self.source.get(self.context.position + 1) {
        Some('*') => match self.source.get(self.context.position + 2) {
          Some('=') => {
            self.context.position += 3;
            Token::DoubleAsteriskEquals
          }
          _ => {
            self.context.position += 2;
            Token::DoubleAsterisk
          }
        },
        Some('=') => {
          self.context.position += 2;
          Token::AsteriskEquals
        }
        _ => {
          self.context.position += 1;
          Token::Asterisk
        }
      },
      '&' => match self.source.get(self.context.position + 1) {
        Some('&') => match self.source.get(self.context.position + 2) {
          Some('=') => {
            self.context.position += 3;
            Token::DoubleAmpersandEquals
          }
          _ => {
            self.context.position += 2;
            Token::DoubleAmpersand
          }
        },
        Some('=') => {
          self.context.position += 2;
          Token::AmpersandEquals
        }
        _ => {
          self.context.position += 1;
          Token::Ampersand
        }
      },
      '|' => match self.source.get(self.context.position + 1) {
        Some('|') => match self.source.get(self.context.position + 2) {
          Some('=') => {
            self.context.position += 3;
            Token::DoubleVerticalLineEquals
          }
          _ => {
            self.context.position += 2;
            Token::DoubleVerticalLine
          }
        },
        Some('=') => {
          self.context.position += 2;
          Token::VerticalLineEquals
        }
        _ => {
          self.context.position += 1;
          Token::VerticalLine
        }
      },
      '^' => match self.source.get(self.context.position + 1) {
        Some('=') => {
          self.context.position += 2;
          Token::CircumflexEquals
        }
        _ => {
          self.context.position += 1;
          Token::Circumflex
        }
      },
      '%' => match self.source.get(self.context.position + 1) {
        Some('=') => {
          self.context.position += 2;
          Token::PercentEquals
        }
        _ => {
          self.context.position += 1;
          Token::Percent
        }
      },
      '{' => simple_punctuator!(Token::LeftCurlyBracket),
      '(' => simple_punctuator!(Token::LeftParenthesis),
      ')' => simple_punctuator!(Token::RightParenthesis),
      '[' => simple_punctuator!(Token::LeftSquareBracket),
      ']' => simple_punctuator!(Token::RightSquareBracket),
      ';' => simple_punctuator!(Token::Semicolon),
      ',' => simple_punctuator!(Token::Comma),
      '~' => simple_punctuator!(Token::Tilde),
      ':' => simple_punctuator!(Token::Colon),
      '#' => simple_punctuator!(Token::NumberSign),
      _ => return Err(syntax_err!()),
    };

    self.context.token = token;
    Ok(())
  }

  fn required_char<I: SliceIndex<[char]>>(&self, position: I) -> Result<&I::Output, ParseError> {
    match self.source.get(position) {
      Some(c) => Ok(c),
      _ => Err(syntax_err!()),
    }
  }

  fn read_word_token(&mut self, start_char: char) -> Result<Token, ParseError> {
    let mut characters = String::new();
    let mut has_unicode_escape = false;
    characters.push(start_char);

    loop {
      match self.source.get(self.context.position) {
        Some('\\') => match self.required_char(self.context.position + 1)? {
          'u' => {
            self.context.position += 2;
            let character = self.read_unicode_escape_sequence()?;

            if is_id_continue(character) {
              has_unicode_escape = true;
              characters.push(character)
            } else {
              return Err(syntax_err!());
            }
          }
          _ => return Err(syntax_err!()),
        },
        Some(c) if is_id_continue(*c) => {
          self.context.position += 1;
          characters.push(*c);
        }
        _ => break,
      }
    }

    let word = if has_unicode_escape {
      Name::from_escaped_string(characters)
    } else {
      Name::from_string(characters)
    };

    Ok(Token::Name(word))
  }
}
