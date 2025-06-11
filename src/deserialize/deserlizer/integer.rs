use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer, token_is_space},
  stream_parser::{Token, TokenInfo},
};

fn to_digit(c: char) -> u32 {
  (c as u32) & 0xF
}
fn to_hexdigit(c: char) -> u32 {
  let cv = c as u32;
  if cv < 0x40 { cv & 0xF } else { (cv & 0xF) + 9 }
}

struct SignedDeserializer<Signed, Unsigned> {
  limb: Unsigned,
  mul: Unsigned,
  started: bool,
  is_neg: bool,
  _phantom: std::marker::PhantomData<Signed>,
}
impl<Signed, Unsigned> Deserializer<Signed> for SignedDeserializer<Signed, Unsigned>
where
  Signed: num_traits::PrimInt + num_traits::Signed + num_traits::NumCast,
  Unsigned: num_traits::PrimInt + num_traits::Unsigned + num_traits::NumCast,
{
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<Signed>, DeserError> {
    match token.info {
      TokenInfo::NumberIntegerDigit | TokenInfo::NumberOct | TokenInfo::NumberBin => {
        self.started = true;
        let d = Unsigned::from(to_digit(token.c)).unwrap();
        self.limb = self.limb.checked_mul(&self.mul).ok_or("integer overflow")?;
        self.limb = self.limb.checked_add(&d).ok_or("integer overflow")?;
        Ok(DeserResult::Continue)
      }
      TokenInfo::NumberIntegerSign => {
        self.started = true;
        self.is_neg = token.c == '-';
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
        self.started = true;
        self.mul = Unsigned::from(16).unwrap();
        Ok(DeserResult::Continue)
      }
      TokenInfo::NumberHex => {
        self.started = true;
        let d = Unsigned::from(to_hexdigit(token.c)).unwrap();
        self.limb = self.limb.checked_mul(&self.mul).ok_or("integer overflow")?;
        self.limb = self.limb.checked_add(&d).ok_or("integer overflow")?;
        Ok(DeserResult::Continue)
      }
      TokenInfo::NumberOctStart => {
        self.started = true;
        self.mul = Unsigned::from(8).unwrap();
        Ok(DeserResult::Continue)
      }
      TokenInfo::NumberBinStart => {
        self.started = true;
        self.mul = Unsigned::from(2).unwrap();
        Ok(DeserResult::Continue)
      }
      _ => {
        if self.started {
          if self.is_neg {
            if self.limb
              >= Unsigned::from(Signed::max_value()).unwrap().add(Unsigned::from(1).unwrap())
            {
              Err("integer overflow".into())
            } else if self.limb == Unsigned::zero() {
              Err("negative zero is not allowed".into())
            } else {
              Ok(DeserResult::CompleteWithRollback(-Signed::from(self.limb).unwrap()))
            }
          } else {
            if self.limb >= Unsigned::from(Signed::max_value()).unwrap() {
              Err("integer overflow".into())
            } else {
              Ok(DeserResult::CompleteWithRollback(Signed::from(self.limb).unwrap()))
            }
          }
        } else {
          if token_is_space(&token) {
            Ok(DeserResult::Continue)
          } else {
            Err("expect integer".into())
          }
        }
      }
    }
  }
}

impl DefaultDeserializable<i8> for i8 {
  fn default_deserializer() -> impl Deserializer<i8> {
    SignedDeserializer::<i8, u8> {
      limb: 0,
      mul: 10,
      started: false,
      is_neg: false,
      _phantom: std::marker::PhantomData,
    }
  }
}
impl DefaultDeserializable<i16> for i16 {
  fn default_deserializer() -> impl Deserializer<i16> {
    SignedDeserializer::<i16, u16> {
      limb: 0,
      mul: 10,
      started: false,
      is_neg: false,
      _phantom: std::marker::PhantomData,
    }
  }
}
impl DefaultDeserializable<i32> for i32 {
  fn default_deserializer() -> impl Deserializer<i32> {
    SignedDeserializer::<i32, u32> {
      limb: 0,
      mul: 10,
      started: false,
      is_neg: false,
      _phantom: std::marker::PhantomData,
    }
  }
}
impl DefaultDeserializable<i64> for i64 {
  fn default_deserializer() -> impl Deserializer<i64> {
    SignedDeserializer::<i64, u64> {
      limb: 0,
      mul: 10,
      started: false,
      is_neg: false,
      _phantom: std::marker::PhantomData,
    }
  }
}
impl DefaultDeserializable<i128> for i128 {
  fn default_deserializer() -> impl Deserializer<i128> {
    SignedDeserializer::<i128, u128> {
      limb: 0,
      mul: 10,
      started: false,
      is_neg: false,
      _phantom: std::marker::PhantomData,
    }
  }
}
impl DefaultDeserializable<isize> for isize {
  fn default_deserializer() -> impl Deserializer<isize> {
    SignedDeserializer::<isize, usize> {
      limb: 0,
      mul: 10,
      started: false,
      is_neg: false,
      _phantom: std::marker::PhantomData,
    }
  }
}

struct UnSignedDeserializer<Unsigned> {
  limb: Unsigned,
  mul: Unsigned,
  started: bool,
}
impl<Unsigned> Deserializer<Unsigned> for UnSignedDeserializer<Unsigned>
where
  Unsigned: num_traits::PrimInt + num_traits::Unsigned + num_traits::NumCast,
{
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<Unsigned>, DeserError> {
    match token.info {
      TokenInfo::NumberIntegerDigit | TokenInfo::NumberOct | TokenInfo::NumberBin => {
        self.started = true;
        let d = Unsigned::from(to_digit(token.c)).unwrap();
        self.limb = self.limb.checked_mul(&self.mul).ok_or("integer overflow")?;
        self.limb = self.limb.checked_add(&d).ok_or("integer overflow")?;
        Ok(DeserResult::Continue)
      }
      TokenInfo::NumberIntegerSign => {
        if token.c == '+' {
          self.started = true;
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
        self.started = true;
        self.mul = Unsigned::from(16).unwrap();
        Ok(DeserResult::Continue)
      }
      TokenInfo::NumberHex => {
        self.started = true;
        let d = Unsigned::from(to_hexdigit(token.c)).unwrap();
        self.limb = self.limb.checked_mul(&self.mul).ok_or("integer overflow")?;
        self.limb = self.limb.checked_add(&d).ok_or("integer overflow")?;
        Ok(DeserResult::Continue)
      }
      TokenInfo::NumberOctStart => {
        self.started = true;
        self.mul = Unsigned::from(8).unwrap();
        Ok(DeserResult::Continue)
      }
      TokenInfo::NumberBinStart => {
        self.started = true;
        self.mul = Unsigned::from(2).unwrap();
        Ok(DeserResult::Continue)
      }
      _ => {
        if self.started {
          Ok(DeserResult::CompleteWithRollback(self.limb))
        } else {
          if token_is_space(&token) {
            Ok(DeserResult::Continue)
          } else {
            Err("expect integer".into())
          }
        }
      }
    }
  }
}
impl DefaultDeserializable<u8> for u8 {
  fn default_deserializer() -> impl Deserializer<u8> {
    UnSignedDeserializer { limb: 0, mul: 10, started: false }
  }
}
impl DefaultDeserializable<u16> for u16 {
  fn default_deserializer() -> impl Deserializer<u16> {
    UnSignedDeserializer { limb: 0, mul: 10, started: false }
  }
}
impl DefaultDeserializable<u32> for u32 {
  fn default_deserializer() -> impl Deserializer<u32> {
    UnSignedDeserializer { limb: 0, mul: 10, started: false }
  }
}
impl DefaultDeserializable<u64> for u64 {
  fn default_deserializer() -> impl Deserializer<u64> {
    UnSignedDeserializer { limb: 0, mul: 10, started: false }
  }
}
impl DefaultDeserializable<u128> for u128 {
  fn default_deserializer() -> impl Deserializer<u128> {
    UnSignedDeserializer { limb: 0, mul: 10, started: false }
  }
}
impl DefaultDeserializable<usize> for usize {
  fn default_deserializer() -> impl Deserializer<usize> {
    UnSignedDeserializer { limb: 0, mul: 10, started: false }
  }
}
