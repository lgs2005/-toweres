use port_oxc_number_parsers::{parse_binary, parse_decimal, parse_hexadecimal, parse_octal};

use crate::{
  bigint::BigInt,
  parser::{
    error::{syntax_err, ParseError, ParseErrorCode},
    Parser,
  },
};

use super::{identifier_utils::is_id_start, parse_err, token::Token};

impl<'r, 'a: 'r> Parser<'r, 'a> {
  pub fn read_zero_starting_literal(&mut self) -> Result<Token, ParseError> {
    self.context.position += 1;

    macro_rules! parse_nondecimal_digits {
      ($pat:pat) => {{
        let mut digits = Vec::<char>::new();

        match self.source.get(self.context.position) {
          Some(digit @ ($pat)) => {
            self.context.position += 1;
            digits.push(*digit);
          }
          _ => return Err(syntax_err!()),
        }

        loop {
          match self.source.get(self.context.position) {
            Some(digit @ ($pat)) => {
              self.context.position += 1;
              digits.push(*digit);
            }
            Some('_') => {
              self.context.position += 1;
            }
            _ => break,
          }
        }

        if matches!(self.source.get(self.context.position - 1), Some('_')) {
          return Err(syntax_err!());
        }

        digits
      }};
    }

    let token = match self.source.get(self.context.position) {
      Some('n') => {
        self.context.position += 1;
        Token::BigIntLiteral(BigInt::new(vec![0u64]))
      }
      Some('b' | 'B') => {
        self.context.position += 1;
        let digits = parse_nondecimal_digits!('0' | '1');

        match self.source.get(self.context.position) {
          Some('n') => {
            self.context.position += 1;
            Token::BigIntLiteral(BigInt::from_binary_str(&digits))
          }
          _ => Token::NumberLiteral(parse_binary(&digits)),
        }
      }
      Some('o' | 'O') => {
        self.context.position += 1;
        let digits = parse_nondecimal_digits!('0'..='7');

        match self.source.get(self.context.position) {
          Some('n') => {
            self.context.position += 1;
            Token::BigIntLiteral(BigInt::from_octal_str(&digits))
          }
          _ => Token::NumberLiteral(parse_octal(&digits)),
        }
      }
      Some('x' | 'X') => {
        self.context.position += 1;
        let digits = parse_nondecimal_digits!('0'..='9' | 'a'..='f' | 'A'..='F');

        match self.source.get(self.context.position) {
          Some('n') => {
            self.context.position += 1;
            Token::BigIntLiteral(BigInt::from_hex_str(&digits))
          }
          _ => Token::NumberLiteral(parse_hexadecimal(&digits)),
        }
      }
      Some(digit @ '0'..='9') => {
        if self.context.flags.strict_mode {
          return Err(parse_err!(ParseErrorCode::StrictOctalLiteral));
        }

        self.context.position += 1;

        let mut digits = Vec::<char>::new();
        if *digit != '0' {
          digits.push(*digit);
        }

        loop {
          match self.source.get(self.context.position) {
            Some(digit @ '0'..='9') => {
              self.context.position += 1;
              if *digit != '0' || digits.len() > 0 {
                digits.push(*digit);
              }
            }
            _ => break,
          }
        }

        if !digits.iter().any(|digit| matches!(*digit, '8' | '9')) {
          Token::NumberLiteral(parse_octal(&digits))
        } else {
          self.read_decimal_literal(Some(digits))?
        }
      }
      _ => Token::NumberLiteral(0f64),
    };

    self.check_end_of_numeric_literal()?;
    Ok(token)
  }

  pub fn read_nonzero_starting_literal(&mut self, digit: char) -> Result<Token, ParseError> {
    self.context.position += 1;

    let mut digits = Vec::<char>::new();
    digits.push(digit);

    loop {
      match self.source.get(self.context.position) {
        Some('_') => {
          self.context.position += 1;
        }
        Some(digit @ '0'..='9') => {
          self.context.position += 1;
          if *digit != '0' || digits.len() > 0 {
            digits.push(*digit);
          }
        }
        _ => break,
      }
    }

    if matches!(self.source.get(self.context.position - 1), Some('_')) {
      return Err(syntax_err!());
    }

    let token = if matches!(self.source.get(self.context.position), Some('n')) {
      self.context.position += 1;
      Token::BigIntLiteral(BigInt::from_decimal_str(&digits))
    } else {
      self.read_decimal_literal(Some(digits))?
    };

    Ok(token)
  }

  pub fn read_decimal_literal(
    &mut self,
    integer_digits: Option<Vec<char>>,
  ) -> Result<Token, ParseError> {
    let numeric_value = match self.source.get(self.context.position) {
      Some('.') => {
        self.context.position += 1;

        let mut digits_str = match integer_digits {
          Some(digits) => String::from_iter(digits),
          None => String::new(),
        };

        digits_str.push('.');

        match self.source.get(self.context.position) {
          Some(digit @ '0'..='9') => {
            self.context.position += 1;
            digits_str.push(*digit);
          }
          _ => return Err(syntax_err!()),
        }

        loop {
          match self.source.get(self.context.position) {
            Some(digit @ '0'..='9') => {
              self.context.position += 1;
              digits_str.push(*digit);
            }
            Some('_') => {
              self.context.position += 1;
            }
            _ => break,
          }
        }

        if matches!(self.source.get(self.context.position - 1), Some('_')) {
          return Err(syntax_err!());
        }

        digits_str.parse::<f64>().unwrap()
      }
      _ => match integer_digits {
        Some(digits) => parse_decimal(&digits),
        None => return Err(syntax_err!()),
      },
    };

    let token = match self.source.get(self.context.position) {
      Some('e' | 'E') => {
        self.context.position += 1;

        let sign: i8 = match self.source.get(self.context.position) {
          Some('+') => {
            self.context.position += 1;
            1
          }
          Some('-') => {
            self.context.position += 1;
            -1
          }
          Some('0'..='9') => 1,
          _ => return Err(syntax_err!()),
        };

        let mut exponent_digits = Vec::<char>::new();

        match self.source.get(self.context.position) {
          Some(digit @ ('0'..='9')) => {
            self.context.position += 1;
            if *digit != '0' {
              exponent_digits.push(*digit)
            }
          }
          _ => return Err(syntax_err!()),
        };

        loop {
          match self.source.get(self.context.position) {
            Some(digit @ ('0'..='9')) => {
              self.context.position += 1;
              if *digit != '0' || exponent_digits.len() > 0 {
                exponent_digits.push(*digit);
              }
            }
            Some('_') => {
              self.context.position += 1;
            }
            _ => break,
          }
        }

        if matches!(self.source.get(self.context.position - 1), Some('_')) {
          return Err(syntax_err!());
        }

        let exponent_value = parse_decimal(&exponent_digits) * sign as f64;

        Token::NumberLiteral(numeric_value * 10f64.powf(exponent_value))
      }
      _ => Token::NumberLiteral(numeric_value),
    };

    self.check_end_of_numeric_literal()?;
    Ok(token)
  }

  fn check_end_of_numeric_literal(&mut self) -> Result<(), ParseError> {
    match self.source.get(self.context.position) {
      Some('0'..='9') => Err(syntax_err!()),
      Some(c) if is_id_start(*c) => Err(syntax_err!()),
      _ => Ok(()),
    }
  }
}
