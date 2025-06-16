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

#[derive(Debug)]
pub struct FloatDeserializer {
  list: String,
  is_neg: bool,
}

macro_rules! float_deserializer {
  ($typ: ty) => {
    impl Deserializer<$typ> for FloatDeserializer {
      fn feed_token(
        &mut self,
        token: crate::stream_parser::Token,
      ) -> Result<DeserResult<$typ>, DeserError> {
        match token.info {
          TokenInfo::NumberIntegerSign => {
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
            self.list.push(token.c);
            Ok(DeserResult::Continue)
          }
          _ => {
            if !self.list.is_empty() {
              if unsafe { self.list.bytes().nth(0).unwrap_unchecked() } == b'0' {
                let mut iter = self.list.bytes();
                iter.next();
                if let Some(b) = iter.next() {
                  match b {
                    b'x' | b'X' => {
                      let mut ret: $typ = 0.0;
                      for c in iter {
                        ret *= 16.0;
                        ret += to_hexdigit(c) as $typ;
                      }
                      return Ok(DeserResult::CompleteWithRollback(if self.is_neg {
                        -ret
                      } else {
                        ret
                      }));
                    }
                    b'o' | b'O' => {
                      let mut ret: $typ = 0.0;
                      for c in iter {
                        ret *= 8.0;
                        ret += (c & 0xF) as $typ;
                      }
                      return Ok(DeserResult::CompleteWithRollback(if self.is_neg {
                        -ret
                      } else {
                        ret
                      }));
                    }
                    b'b' | b'B' => {
                      let mut ret: $typ = 0.0;
                      for c in iter {
                        ret *= 2.0;
                        ret += (c & 0xF) as $typ;
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

              match self.list.parse::<$typ>() {
                Ok(val) => {
                  Ok(DeserResult::CompleteWithRollback(if self.is_neg { -val } else { val }))
                }
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
    impl DefaultDeserializable<$typ> for $typ {
      type DefaultDeserializer = FloatDeserializer;
      fn default_deserializer() -> FloatDeserializer {
        FloatDeserializer { list: String::default(), is_neg: false }
      }
    }
  };
}

float_deserializer!(f32);
float_deserializer!(f64);
