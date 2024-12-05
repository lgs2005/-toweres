use port_oxc_number_parsers::hex_digit_value;

use crate::parser::{
  error::{ParseError, ParseErrorCode},
  parse_err, Parser,
};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_string_escape_sequence(&mut self) -> Result<Option<char>, ParseError> {
    macro_rules! simple_escape {
      ($character:expr) => {{
        self.context.position += 1;
        Some($character)
      }};
    }

    let escape_char = match self.required_char(self.context.position)? {
      '\u{000A}' | '\u{2028}' | '\u{2029}' => {
        self.context.position += 1;
        None
      }
      '\u{000D}' => {
        match self.source.get(self.context.position + 1) {
          Some('\u{000A}') => self.context.position += 2,
          _ => self.context.position += 1,
        }
        None
      }
      '0' => match self.required_char(self.context.position + 1)? {
        '0'..='7' => Some(self.read_legacy_octal_escape_sequence()?),
        '8' | '9' => {
          if self.context.flags.strict_mode {
            return Err(parse_err!(ParseErrorCode::StrictOctalEscape));
          } else {
            self.context.position += 1;
            Some('\0')
          }
        }
        _ => {
          self.context.position += 1;
          Some('\0')
        }
      },
      '1'..='7' => Some(self.read_legacy_octal_escape_sequence()?),
      c @ ('8' | '9') => {
        if self.context.flags.strict_mode {
          return Err(parse_err!(ParseErrorCode::StrictOctalEscape));
        } else {
          let c = *c;
          self.context.position += 1;
          Some(c)
        }
      }
      'x' => {
        self.context.position += 1;
        Some(self.read_double_digit_hex_escape_sequence()?)
      }
      'u' => {
        self.context.position += 1;
        Some(self.read_unicode_escape_sequence()?)
      }
      'b' => simple_escape!('\u{0008}'),
      't' => simple_escape!('\u{0009}'),
      'n' => simple_escape!('\u{000A}'),
      'v' => simple_escape!('\u{000B}'),
      'f' => simple_escape!('\u{000C}'),
      'r' => simple_escape!('\u{000D}'),
      c => {
        let c = *c;
        self.context.position += 1;
        Some(c)
      }
    };

    Ok(escape_char)
  }

  pub fn read_unicode_escape_sequence(&mut self) -> Result<char, ParseError> {
    match self.required_char(self.context.position)? {
      '{' => {
        self.context.position += 1;
        let start_index = self.context.position;

        loop {
          match self.required_char(self.context.position)? {
            '}' => {
              self.context.position += 1;
              break;
            }
            '0'..='9' | 'a'..='z' | 'A'..='Z' => self.context.position += 1,
            _ => return Err(parse_err!(ParseErrorCode::InvalidEscape)),
          }
        }

        let digits = &self.source[start_index..(self.context.position - 1)];

        if digits.len() > 6 {
          return Err(parse_err!(ParseErrorCode::InvalidEscape));
        }

        let mut codepoint = 0u64;
        for digit in digits {
          codepoint <<= 4;
          codepoint |= hex_digit_value(*digit);
        }

        if codepoint > (u32::MAX as u64) {
          return Err(parse_err!(ParseErrorCode::InvalidUnicode));
        }

        match char::from_u32(codepoint as u32) {
          None => Err(parse_err!(ParseErrorCode::InvalidUnicode)),
          Some(c) => Ok(c),
        }
      }
      _ => self.read_double_digit_hex_escape_sequence(),
    }
  }

  fn read_double_digit_hex_escape_sequence(&mut self) -> Result<char, ParseError> {
    let first_digit = *self.required_char(self.context.position)?;
    let second_digit = *self.required_char(self.context.position)?;

    if !(is_hex_digit(first_digit) && is_hex_digit(second_digit)) {
      return Err(parse_err!(ParseErrorCode::InvalidEscape));
    }

    let codepoint = (hex_digit_value(first_digit) << 4) | hex_digit_value(second_digit);

    match char::from_u32(codepoint as u32) {
      None => Err(parse_err!(ParseErrorCode::InvalidUnicode)),
      Some(c) => {
        self.context.position += 2;
        Ok(c)
      }
    }
  }

  fn read_legacy_octal_escape_sequence(&mut self) -> Result<char, ParseError> {
    if self.context.flags.strict_mode {
      return Err(parse_err!(ParseErrorCode::StrictOctalEscape));
    }

    let start_index = self.context.position;
    self.context.position += match self.required_char(self.context.position)? {
      '0'..='3' => match self.required_char(self.context.position + 1)? {
        '0'..='7' => match self.required_char(self.context.position + 2)? {
          '0'..='7' => 3,
          _ => 2,
        },
        _ => 1,
      },
      '4'..='7' => match self.required_char(self.context.position + 1)? {
        '0'..='7' => 2,
        _ => 1,
      },
      _ => unreachable!(),
    };

    let digits = &self.source[start_index..self.context.position];
    let mut codepoint = 0u32;

    for digit in digits {
      codepoint <<= 3;
      codepoint |= *digit as u32 & 0xF;
    }

    match char::from_u32(codepoint) {
      None => Err(parse_err!(ParseErrorCode::InvalidUnicode)),
      Some(c) => Ok(c),
    }
  }
}

fn is_hex_digit(c: char) -> bool {
  matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F')
}
