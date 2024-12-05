#[derive(Debug, Clone)]
pub struct BigInt {
  parts: Vec<u64>,
}

impl BigInt {
  pub fn new(parts: Vec<u64>) -> Self {
    Self { parts }
  }

  pub fn from_decimal_str(chars: &[char]) -> Self {
    let mut self_ = BigInt::new(Vec::new());
    for digit in chars {
      let mut carry = *digit as u64 & 15;

      for i in 0..self_.parts.len() {
        (self_.parts[i], carry) = self_.parts[i].carrying_mul(10, carry);
      }

      if carry > 0 {
        self_.parts.push(carry);
      }
    }
    self_
  }

  pub fn from_octal_str(chars: &[char]) -> Self {
    let mut parts = Vec::<u64>::with_capacity(chars.len() * 3 / 64 + 1);

    for (char_index, c) in chars.iter().enumerate().rev() {
      let value = *c as u64 & 7;
      let bit_index = char_index * 3;
      let part_index = bit_index / 64;
      let part_bit_index = bit_index % 64;
      let overflow = part_bit_index + 3 - 64;

      parts[part_index] |= value << part_bit_index;

      if overflow > 0 {
        parts[part_index + 1] |= value >> overflow;
      }
    }

    BigInt::new(parts)
  }

  pub fn from_hex_str(chars: &[char]) -> Self {
    let mut parts = Vec::<u64>::with_capacity(chars.len() * 16 / 64 + 1);

    for (char_index, c) in chars.iter().enumerate().rev() {
      let value = if (*c as u8) < b'A' {
        *c as u64 & 15
      } else {
        (*c as u64 & 15) + 9
      };

      let bit_index = char_index * 16;

      parts[bit_index / 64] |= value << bit_index % 64;
    }

    BigInt::new(parts)
  }

  pub fn from_binary_str(chars: &[char]) -> Self {
    let mut parts = Vec::<u64>::with_capacity(chars.len() / 64 + 1);

    for (char_index, c) in chars.iter().enumerate().rev() {
      parts[char_index / 64] |= (*c as u64 & 1) << char_index % 64;
    }

    BigInt::new(parts)
  }
}
