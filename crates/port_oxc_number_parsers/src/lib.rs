/*
This file contains code copied and modified from the JavaScript Oxidation Compiler project.
Original code: https://github.com/oxc-project/oxc/blob/810671a4a06e2702832d41423fbfe593068fee72/crates/oxc_parser/src/lexer/number.rs

MIT License

Copyright (c) 2024-present VoidZero Inc. & Contributors
Copyright (c) 2023 Boshen

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

pub fn hex_digit_value(c: char) -> u64 {
  if c >= 'a' {
    (c as u64 & 0xF) + 9
  } else {
    c as u64 & 0xF
  }
}

pub fn parse_hexadecimal(digits: &[char]) -> f64 {
  if digits.len() > 64 {
    let mut result = 0f64;
    for digit in digits {
      result = result.mul_add(16f64, hex_digit_value(*digit) as f64)
    }
    result
  } else {
    let mut result = 0u64;
    for digit in digits {
      result <<= 4;
      result |= hex_digit_value(*digit);
    }
    result as f64
  }
}

pub fn parse_decimal(digits: &[char]) -> f64 {
  if digits.len() > 19 {
    String::from_iter(digits).parse::<f64>().unwrap()
  } else {
    let mut result = 0u64;
    for digit in digits {
      result *= 10;
      result += *digit as u64 & 0xF
    }
    result as f64
  }
}

pub fn parse_octal(digits: &[char]) -> f64 {
  if digits.len() > 64 {
    let mut result = 0f64;
    for digit in digits {
      result = result.mul_add(8f64, (*digit as u8 & 0xF) as f64)
    }
    result
  } else {
    let mut result = 0u64;
    for digit in digits {
      result <<= 3;
      result |= *digit as u64 & 0xF;
    }
    result as f64
  }
}

pub fn parse_binary(digits: &[char]) -> f64 {
  if digits.len() > 64 {
    let mut result = 0f64;
    for digit in digits {
      result = result.mul_add(2f64, (*digit as u8 & 1) as f64)
    }
    result
  } else {
    let mut result = 0u64;
    for digit in digits {
      result <<= 1;
      result |= *digit as u64 & 1;
    }
    result as f64
  }
}
