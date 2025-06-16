use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer},
  stream_parser::{Token, TokenInfo},
};

#[derive(Debug)]
pub struct IntegerDeserializer {
  list: String,
  radix: u32,
}

macro_rules! signed_deserializer {
  ($typ: ty) => {
    impl Deserializer<$typ> for IntegerDeserializer {
      fn feed_token(&mut self, token: Token) -> Result<DeserResult<$typ>, DeserError> {
        match token.info {
          TokenInfo::NumberIntegerDigit
          | TokenInfo::NumberOct
          | TokenInfo::NumberBin
          | TokenInfo::NumberHex => {
            self.list.push(token.c);
            Ok(DeserResult::Continue)
          }
          TokenInfo::NumberIntegerSign => {
            self.list.push(token.c);
            Ok(DeserResult::Continue)
          }
          TokenInfo::NumberFractionDigit
          | TokenInfo::NumberExponentDigit
          | TokenInfo::NumberFractionStart
          | TokenInfo::NumberExponentStart
          | TokenInfo::NumberExponentSign => Err("not an integer".into()),
          TokenInfo::NumberNan(_, _) => Err("NaN is not an integer".into()),
          TokenInfo::NumberInfinity(_, _) => Err("Infinity is not an integer".into()),
          TokenInfo::NumberHexStart => {
            self.list.clear();
            self.radix = 16;
            Ok(DeserResult::Continue)
          }
          TokenInfo::NumberOctStart => {
            self.list.clear();
            self.radix = 8;
            Ok(DeserResult::Continue)
          }
          TokenInfo::NumberBinStart => {
            self.list.clear();
            self.radix = 2;
            Ok(DeserResult::Continue)
          }
          _ => {
            if !self.list.is_empty() {
              match <$typ>::from_str_radix(&self.list, self.radix) {
                Ok(val) => Ok(DeserResult::CompleteWithRollback(val)),
                Err(e) => Err(e.into()),
              }
            } else {
              if token.is_space() {
                Ok(DeserResult::Continue)
              } else {
                Err("expect integer".into())
              }
            }
          }
        }
      }
    }
    impl DefaultDeserializable<$typ> for $typ {
      type DefaultDeserializer = IntegerDeserializer;
      fn default_deserializer() -> IntegerDeserializer {
        IntegerDeserializer { list: String::new(), radix: 10 }
      }
    }
  };
}

signed_deserializer! {i8}
signed_deserializer! {i16}
signed_deserializer! {i32}
signed_deserializer! {i64}
signed_deserializer! {i128}
signed_deserializer! {isize}

macro_rules! unsigned_deserializer {
  ($typ: ty) => {
    impl Deserializer<$typ> for IntegerDeserializer {
      fn feed_token(&mut self, token: Token) -> Result<DeserResult<$typ>, DeserError> {
        match token.info {
          TokenInfo::NumberIntegerDigit
          | TokenInfo::NumberOct
          | TokenInfo::NumberBin
          | TokenInfo::NumberHex => {
            self.list.push(token.c);
            Ok(DeserResult::Continue)
          }
          TokenInfo::NumberIntegerSign => {
            if token.c == '+' {
              self.list.push(token.c);
              Ok(DeserResult::Continue)
            } else {
              Err("unsigned integer cannot be negative".into())
            }
          }
          TokenInfo::NumberFractionDigit
          | TokenInfo::NumberExponentDigit
          | TokenInfo::NumberFractionStart
          | TokenInfo::NumberExponentStart
          | TokenInfo::NumberExponentSign => Err("not an integer".into()),
          TokenInfo::NumberNan(_, _) => Err("NaN is not an integer".into()),
          TokenInfo::NumberInfinity(_, _) => Err("Infinity is not an integer".into()),
          TokenInfo::NumberHexStart => {
            self.list.clear();
            self.radix = 16;
            Ok(DeserResult::Continue)
          }
          TokenInfo::NumberOctStart => {
            self.list.clear();
            self.radix = 8;
            Ok(DeserResult::Continue)
          }
          TokenInfo::NumberBinStart => {
            self.list.clear();
            self.radix = 2;
            Ok(DeserResult::Continue)
          }
          _ => {
            if !self.list.is_empty() {
              match <$typ>::from_str_radix(&self.list, self.radix) {
                Ok(val) => Ok(DeserResult::CompleteWithRollback(val)),
                Err(e) => Err(e.into()),
              }
            } else {
              if token.is_space() {
                Ok(DeserResult::Continue)
              } else {
                Err("expect integer".into())
              }
            }
          }
        }
      }
    }
    impl DefaultDeserializable<$typ> for $typ {
      type DefaultDeserializer = IntegerDeserializer;
      fn default_deserializer() -> IntegerDeserializer {
        IntegerDeserializer { list: String::new(), radix: 10 }
      }
    }
  };
}

unsigned_deserializer! {u8}
unsigned_deserializer! {u16}
unsigned_deserializer! {u32}
unsigned_deserializer! {u64}
unsigned_deserializer! {u128}
unsigned_deserializer! {usize}
