use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer, token_is_space},
  stream_parser::TokenInfo,
};

fn to_hexdigit(c: u8) -> u8 {
  match c {
    0x30..0x39 => c & 0xF,
    _ => (c & 0xF) + 9,
  }
}

struct F64Deserializer {
  list: String,
  is_neg: bool,
  started: bool,
}
impl Deserializer<f64> for F64Deserializer {
  fn feed_token(
    &mut self,
    token: crate::stream_parser::Token,
  ) -> Result<DeserResult<f64>, DeserError> {
    match token.info {
      TokenInfo::NumberIntegerSign => {
        self.started = true;
        self.is_neg = token.c == '-';
        Ok(DeserResult::Continue)
      }
      TokenInfo::NumberIntegerDigit
      | TokenInfo::NumberOct
      | TokenInfo::NumberBin
      | TokenInfo::NumberFractionDigit
      | TokenInfo::NumberExponentDigit
      | TokenInfo::NumberFractionStart
      | TokenInfo::NumberExponentStart
      | TokenInfo::NumberExponentSign
      | TokenInfo::NumberNan(_, _)
      | TokenInfo::NumberInfinity(_, _)
      | TokenInfo::NumberHexStart
      | TokenInfo::NumberHex
      | TokenInfo::NumberOctStart
      | TokenInfo::NumberBinStart => {
        self.started = true;
        self.list.push(token.c);
        Ok(DeserResult::Continue)
      }
      _ => {
        if self.started {
          if unsafe { self.list.bytes().nth(0).unwrap_unchecked() } == b'0' {
            let mut iter = self.list.bytes();
            iter.next();
            if let Some(b) = iter.next() {
              match b {
                b'x' | b'X' => {
                  let mut ret: f64 = 0.0;
                  for c in iter {
                    ret *= 16.0;
                    ret += to_hexdigit(c) as f64;
                  }
                  return Ok(DeserResult::CompleteWithRollback(if self.is_neg {
                    -ret
                  } else {
                    ret
                  }));
                }
                b'o' | b'O' => {
                  let mut ret: f64 = 0.0;
                  for c in iter {
                    ret *= 8.0;
                    ret += to_hexdigit(c) as f64;
                  }
                  return Ok(DeserResult::CompleteWithRollback(if self.is_neg {
                    -ret
                  } else {
                    ret
                  }));
                }
                b'b' | b'B' => {
                  let mut ret: f64 = 0.0;
                  for c in iter {
                    ret *= 2.0;
                    ret += to_hexdigit(c) as f64;
                  }
                  return Ok(DeserResult::CompleteWithRollback(if self.is_neg {
                    -ret
                  } else {
                    ret
                  }));
                }
                _ => {}
              }
            }
          }

          match self.list.parse::<f64>() {
            Ok(val) => Ok(DeserResult::CompleteWithRollback(if self.is_neg { -val } else { val })),
            Err(e) => Err(format!("parse float error: {}", e).into()),
          }
        } else {
          if token_is_space(&token) {
            Ok(DeserResult::Continue)
          } else {
            Err("expect number".into())
          }
        }
      }
    }
  }
}
impl DefaultDeserializable<f64> for f64 {
  fn default_deserializer() -> impl Deserializer<f64> {
    F64Deserializer { list: String::default(), started: false, is_neg: false }
  }
}

struct F32Deserializer {
  deserializer: F64Deserializer,
}

impl Deserializer<f32> for F32Deserializer {
  fn feed_token(
    &mut self,
    token: crate::stream_parser::Token,
  ) -> Result<DeserResult<f32>, DeserError> {
    Ok(self.deserializer.feed_token(token)?.map(|v| v as f32))
  }
}
impl DefaultDeserializable<f32> for f32 {
  fn default_deserializer() -> impl Deserializer<f32> {
    F32Deserializer {
      deserializer: F64Deserializer { list: String::default(), started: false, is_neg: false },
    }
  }
}
