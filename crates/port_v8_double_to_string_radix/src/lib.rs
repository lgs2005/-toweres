/*
Rust translation of DoubleToRadixCString from the v8 project.
Original code: https://github.com/v8/v8/blob/c3e48a7c58d9a88cb46848b59fb1f621c72a9606/src/numbers/conversions.cc#L1231

Copyright 2006-2011, the V8 project authors. All rights reserved.
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above
      copyright notice, this list of conditions and the following
      disclaimer in the documentation and/or other materials provided
      with the distribution.
    * Neither the name of Google Inc. nor the names of its
      contributors may be used to endorse or promote products derived
      from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

#![feature(float_next_up_down)]

pub fn double_to_string_radix(mut value: f64, radix: u8) -> String {
  const CHARS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
  const BUFFER_SIZE: usize = 2200;

  let radixf = radix as f64;
  let mut buffer = [0u8; BUFFER_SIZE];
  let mut integer_cursor = BUFFER_SIZE / 2;
  let mut fraction_cursor = integer_cursor;

  let negative = value < 0.0;
  if negative {
    value = -value;
  }

  let mut integer = value.floor();
  let mut fraction = value.fract();
  let mut delta = f64::max(0f64.next_up(), 0.5 * (value.next_up() - value));

  if fraction >= delta {
    buffer[fraction_cursor] = b'.';
    fraction_cursor += 1;

    loop {
      fraction *= radixf;
      delta *= radixf;

      let mut digit = fraction as u8;
      buffer[fraction_cursor] = CHARS[digit as usize];
      fraction_cursor += 1;
      fraction -= digit as f64;

      if fraction > 0.5 || (fraction == 0.5 && (digit & 1) != 0) {
        if fraction + delta > 1.0 {
          loop {
            fraction_cursor -= 1;
            if fraction_cursor == BUFFER_SIZE / 2 {
              integer += 1.0;
              break;
            }

            let c = buffer[fraction_cursor];
            digit = if c > b'9' { c - b'a' + 10 } else { c - b'0' };

            if digit + 1 < radix {
              buffer[fraction_cursor] = CHARS[digit as usize + 1];
              fraction_cursor += 1;
              break;
            }
          }
          break;
        }
      }

      if fraction < delta {
        break;
      }
    }
  }

  while (((integer / radixf).to_bits() >> 52) & 0x7FF) as i32 - 1023 > 0 {
    integer /= radixf;
    integer_cursor -= 1;
    buffer[integer_cursor] = b'0';
  }

  loop {
    let remainder = integer % radixf;
    integer_cursor -= 1;
    buffer[integer_cursor] = CHARS[remainder as usize];
    integer = (integer - remainder) / radixf;
    if integer <= 0.0 {
      break;
    }
  }

  if negative {
    integer_cursor -= 1;
    buffer[integer_cursor] = b'-';
  }

  String::from_iter(
    buffer[integer_cursor..fraction_cursor]
      .iter()
      .map(|c| *c as char),
  )
}
