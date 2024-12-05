use std::num::FpCategory;

pub fn es_number_to_string(value: f64, radix: u8) -> String {
  assert!(radix >= 2 && radix <= 36);

  match value.classify() {
    FpCategory::Nan => String::from("NaN"),
    FpCategory::Zero => String::from("0"),
    FpCategory::Infinite => {
      if value.is_sign_positive() {
        String::from("Infinity")
      } else {
        String::from("-Infinity")
      }
    }
    _ => {
      if radix != 10 {
        return port_v8_double_to_string_radix::double_to_string_radix(value, radix);
      } else {
        let (mut significant, rep_exponent, sign) = port_dragonbox::to_decimal(value);

        let mut significant_digits = significant.ilog10() + 1;
        let mut exponent = significant_digits as i32 + rep_exponent;

        let mut buf = [0u8; 64];
        let mut cursor = 0usize;

        if exponent >= -5 && exponent <= 21 {
          if exponent >= significant_digits as i32 {
            while exponent > 0 {
              buf[cursor] = b'0';
              cursor += 1;
              exponent -= 1;
            }
            while significant_digits > 0 {
              buf[cursor] = b'0' + (significant % 10) as u8;
              cursor += 1;
              significant /= 10;
              significant_digits -= 1;
            }
          } else if exponent > 0 {
            while significant_digits > exponent as u32 {
              buf[cursor] = b'0' + (significant % 10) as u8;
              cursor += 1;
              significant /= 10;
              significant_digits -= 1;
            }
            buf[cursor] = b'.';
            cursor += 1;
            while significant_digits > 0 {
              buf[cursor] = b'0' + (significant % 10) as u8;
              cursor += 1;
              significant /= 10;
              significant_digits -= 1;
            }
          } else {
            while significant_digits > 0 {
              buf[cursor] = b'0' + (significant % 10) as u8;
              cursor += 1;
              significant /= 10;
              significant_digits -= 1;
            }
            while exponent < 0 {
              buf[cursor] = b'0';
              cursor += 1;
              exponent += 1;
            }
            buf[cursor] = b'.';
            buf[cursor + 1] = b'0';
            cursor += 2;
          }
        } else {
          let mut exponent_abs = (exponent - 1).unsigned_abs();
          loop {
            buf[cursor] = b'0' + (exponent_abs % 10) as u8;
            cursor += 1;
            if exponent_abs < 10 {
              break;
            }
            exponent_abs /= 10;
          }

          buf[cursor] = if exponent < 0 { b'-' } else { b'+' };
          buf[cursor + 1] = b'e';
          cursor += 2;

          if significant_digits > 1 {
            loop {
              buf[cursor] = b'0' + (significant % 10) as u8;
              cursor += 1;
              significant /= 10;
              if significant < 10 {
                break;
              }
            }

            buf[cursor] = b'.';
            cursor += 1;
          }

          buf[cursor] = b'0' + significant as u8;
          cursor += 1;
        }

        if sign {
          buf[cursor] = b'-';
          cursor += 1;
        }

        String::from_iter(buf[0..cursor].iter().rev().map(|c| *c as char))
      }
    }
  }
}
