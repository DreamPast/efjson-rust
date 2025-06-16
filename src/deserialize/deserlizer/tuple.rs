use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer, token_is_space},
  stream_parser::{Token, TokenInfo},
};

use std::marker::PhantomData;

macro_rules! tuple_wont_end {
  ($self:expr) => {
    Err("the length of array is not enough".into())
  };
}
macro_rules! tuple_should_end {
  ($self:expr) => {
    Ok(DeserResult::Complete(unsafe { $self.ret.assume_init_read() }))
  };
}

macro_rules! tuple_stage {
  ($self:expr, $token:expr, $SubDeserializer:ident, $Receiver:ident, $Type:ident, $index:tt, $end_block:block) => {{
    if $self.stage == -1 {
      if token_is_space(&$token) {
        return Ok(DeserResult::Continue);
      } else {
        match $token.info {
          TokenInfo::ArrayNext => unreachable!(),
          TokenInfo::ArrayEnd => return Err("the length of array is not enough".into()),
          _ => {
            $self.subdeser = $SubDeserializer::$Receiver(
              <$Type as DefaultDeserializable<$Type>>::default_deserializer(),
              PhantomData,
            );
            $self.stage = 0;
          }
        }
      }
    }
    if $self.stage == 1 {
      return if token_is_space(&$token) {
        Ok(DeserResult::Continue)
      } else {
        match $token.info {
          TokenInfo::ArrayNext => {
            $self.stage = -1;
            $self.index = $index + 1;
            Ok(DeserResult::Continue)
          }
          TokenInfo::ArrayEnd => $end_block,
          _ => unreachable!(),
        }
      };
    }
    let $SubDeserializer::$Receiver(subdeser, _) = &mut $self.subdeser else { unreachable!() };
    match subdeser.feed_token($token)? {
      DeserResult::Continue => Ok(DeserResult::Continue),
      DeserResult::Complete(r) => {
        unsafe { std::ptr::addr_of_mut!((*$self.ret.as_mut_ptr()).$index).write(r) };
        $self.stage = 1;
        Ok(DeserResult::Continue)
      }
      DeserResult::CompleteWithRollback(r) => {
        unsafe { std::ptr::addr_of_mut!((*$self.ret.as_mut_ptr()).$index).write(r) };
        // $self.stage = 1;
        if token_is_space(&$token) {
          Ok(DeserResult::Continue)
        } else {
          match $token.info {
            TokenInfo::ArrayNext => {
              $self.stage = -1;
              $self.index = $index + 1;
              Ok(DeserResult::Continue)
            }
            TokenInfo::ArrayEnd => $end_block,
            _ => unreachable!(),
          }
        }
      }
    }
  }};
}
macro_rules! tuple_stage_start {
  ($self:expr, $token:expr) => {
    if token_is_space(&$token) {
      Ok(DeserResult::Continue)
    } else if matches!($token.info, TokenInfo::ArrayStart) {
      $self.stage = -1;
      $self.index = 0;
      Ok(DeserResult::Continue)
    } else {
      Err("expect an array".into())
    }
  };
}
macro_rules! tuple_stage_end {
  ($self:expr, $token:expr) => {{
    if (token_is_space(&$token)) {
      Ok(DeserResult::Continue)
    } else {
      match $token.info {
        TokenInfo::ArrayNext => unreachable!(),
        TokenInfo::ArrayEnd => Ok(DeserResult::Complete(unsafe { $self.ret.assume_init_read() })),
        _ => Err("array length exceeds requirement".into()),
      }
    }
  }};
}

macro_rules! define_tuple_deserializer {
  ($Sub:ident, $Root:ident, $($T:ident, $R:ident),*) => {
    enum $Sub<$($R),*, $($T),*>
    where $($R: Deserializer<$T>),*
    {
      None,
      $($R($R, PhantomData<$T>),)*
    }
    pub struct $Root<$($T),*>
    where $($T: DefaultDeserializable<$T>),*
    {
      stage: i32,
      index: i32,
      subdeser: $Sub<
        $(<$T as DefaultDeserializable<$T>>::DefaultDeserializer),*,
        $($T),*
      >,
      ret: std::mem::MaybeUninit<($($T),*)>,
    }
  };
}
macro_rules! create_tuple {
  ($Sub:ident, $Root:ident, $TT:ident, $RR:ident, $ii: tt, $($T:ident, $R:ident, $i: tt),*) => {
    define_tuple_deserializer! { $Sub, $Root, $($T, $R),*, $TT, $RR }
    impl<$($T),*, $TT> Deserializer<($($T),*, $TT)> for $Root<$($T),*, $TT>
    where $($T: DefaultDeserializable<$T>),*, $TT: DefaultDeserializable<$TT>
    {
      fn feed_token(&mut self, token: Token) -> Result<DeserResult<($($T),*, $TT)>, DeserError> {
        match self.index {
          -1 => tuple_stage_start!(self, token),
          $($i => tuple_stage!(self, token, $Sub, $R, $T, $i, { tuple_wont_end!(self) })),*,
          $ii => tuple_stage!(self, token, $Sub, $RR, $TT, $ii, { tuple_should_end!(self) }),
          val if val == $ii + 1 => tuple_stage_end!(self, token),
          _ => unreachable!(),
        }
      }
    }
    impl<$($T),*, $TT> DefaultDeserializable<($($T),*, $TT)> for ($($T),*, $TT)
    where $($T: DefaultDeserializable<$T>),*, $TT: DefaultDeserializable<$TT>
    {
      type DefaultDeserializer = $Root<$($T),*, $TT>;
      fn default_deserializer() -> Self::DefaultDeserializer {
        $Root {
          stage: 0,
          index: -1,
          subdeser: $Sub::None,
          ret: std::mem::MaybeUninit::uninit(),
        }
      }
    }
    impl<$($T),*, $TT> Drop for $Root<$($T),*, $TT>
    where $($T: DefaultDeserializable<$T>),*, $TT: DefaultDeserializable<$TT>
    {
      fn drop(&mut self) {
        $(
          if self.index > $i || (self.index == $i && self.stage == 1) {
            std::mem::drop(unsafe { std::ptr::addr_of_mut!((*self.ret.as_mut_ptr()).$i).read() });
          }
        )*
        if self.index > $ii || (self.index == $ii && self.stage == 1) {
          std::mem::drop(unsafe { std::ptr::addr_of_mut!((*self.ret.as_mut_ptr()).$ii).read() });
        }
      }
    }
  };
}

create_tuple! {Tuple2Subdeserializer, Tuple2Deserializer, T2, R2, 1,
  T1, R1, 0
}
create_tuple! {Tuple3Subdeserializer, Tuple3Deserializer, T3, R3, 2,
  T1, R1, 0, T2, R2, 1
}
create_tuple! {Tuple4Subdeserializer, Tuple4Deserializer, T4, R4, 3,
  T1, R1, 0, T2, R2, 1, T3, R3, 2
}
create_tuple! {Tuple5Subdeserializer, Tuple5Deserializer, T5, R5, 4,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3
}
create_tuple! {Tuple6Subdeserializer, Tuple6Deserializer, T6, R6, 5,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4
}
create_tuple! {Tuple7Subdeserializer, Tuple7Deserializer, T7, R7, 6,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4, T6, R6, 5
}
create_tuple! {Tuple8Subdeserializer, Tuple8Deserializer, T8, R8, 7,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4, T6, R6, 5, T7, R7, 6
}
create_tuple! {Tuple9Subdeserializer, Tuple9Deserializer, T9, R9, 8,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4, T6, R6, 5, T7, R7, 6, T8, R8, 7
}
create_tuple! {Tuple10Subdeserializer, Tuple10Deserializer, T10, R10, 9,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4, T6, R6, 5, T7, R7, 6, T8, R8, 7,
  T9, R9, 8
}
create_tuple! {Tuple11Subdeserializer, Tuple11Deserializer, T11, R11, 10,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4, T6, R6, 5, T7, R7, 6, T8, R8, 7,
  T9, R9, 8, T10, R10, 9
}
create_tuple! {Tuple12Subdeserializer, Tuple12Deserializer, T12, R12, 11,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4, T6, R6, 5, T7, R7, 6, T8, R8, 7,
  T9, R9, 8, T10, R10, 9, T11, R11, 10
}
create_tuple! {Tuple13Subdeserializer, Tuple13Deserializer, T13, R13, 12,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4, T6, R6, 5, T7, R7, 6, T8, R8, 7,
  T9, R9, 8, T10, R10, 9, T11, R11, 10, T12, R12, 11
}
create_tuple! {Tuple14Subdeserializer, Tuple14Deserializer, T14, R14, 13,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4, T6, R6, 5, T7, R7, 6, T8, R8, 7,
  T9, R9, 8, T10, R10, 9, T11, R11, 10, T12, R12, 11, T13, R13, 12
}
create_tuple! {Tuple15Subdeserializer, Tuple15Deserializer, T15, R15, 14,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4, T6, R6, 5, T7, R7, 6, T8, R8, 7,
  T9, R9, 8, T10, R10, 9, T11, R11, 10, T12, R12, 11, T13, R13, 12, T14, R14, 13
}
create_tuple! {Tuple16Subdeserializer, Tuple16Deserializer, T16, R16, 15,
  T1, R1, 0, T2, R2, 1, T3, R3, 2, T4, R4, 3, T5, R5, 4, T6, R6, 5, T7, R7, 6, T8, R8, 7,
  T9, R9, 8, T10, R10, 9, T11, R11, 10, T12, R12, 11, T13, R13, 12, T14, R14, 13, T15, R15, 14
}
