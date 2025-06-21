use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer},
  stream_parser::{Token, TokenInfo},
};
use std::marker::PhantomData;

macro_rules! tuple_stage {
  ($self:expr, $token:expr,
    $SubDeserializer:ident, $Receiver:ident, $Type:ident,
    $index:tt, $not_enough_block:block, $end_block:block
  ) => {{
    if $self.stage == -1 {
      if $token.is_space() {
        return Ok(DeserResult::Continue);
      } else {
        match $token.info {
          TokenInfo::ArrayNext => unreachable!(),
          TokenInfo::ArrayEnd => return $not_enough_block,
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
      return if $token.is_space() {
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
        if $token.is_space() {
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
    if $token.is_space() {
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
    if ($token.is_space()) {
      Ok(DeserResult::Continue)
    } else {
      match $token.info {
        TokenInfo::ArrayNext => unreachable!(),
        TokenInfo::ArrayEnd => {
          $self.index = -1;
          Ok(DeserResult::Complete(unsafe { $self.ret.assume_init_read() }))
        }
        _ => Err("array length exceeds requirement".into()),
      }
    }
  }};
}

macro_rules! define_tuple_deserializer {
  ($SubDeserializer:ident, $Deserializer:ident, $($T:ident, $R:ident),*) => {
    #[derive(Debug)]
    enum $SubDeserializer<$($R),*, $($T),*>
    where $($R: Deserializer<$T>),*
    {
      None,
      $($R($R, PhantomData<$T>),)*
    }
    pub struct $Deserializer<$($T),*>
    where $($T: DefaultDeserializable<$T>),*
    {
      stage: i32,
      index: i32,
      subdeser: $SubDeserializer<
        $(<$T as DefaultDeserializable<$T>>::DefaultDeserializer),*,
        $($T),*
      >,
      ret: std::mem::MaybeUninit<($($T),*,)>,
    }
  };
}
macro_rules! create_tuple {
  ($SubDeserializer:ident, $Deserializer:ident, $TT:ident, $RR:ident, $ii:tt, $($T:ident, $R:ident, $i:tt),*) => {
    define_tuple_deserializer! { $SubDeserializer, $Deserializer, $($T, $R),*, $TT, $RR }
    impl<$($T),*, $TT> Deserializer<($($T),*, $TT)> for $Deserializer<$($T),*, $TT>
    where $($T: DefaultDeserializable<$T>),*, $TT: DefaultDeserializable<$TT>
    {
      fn feed_token(&mut self, token: Token) -> Result<DeserResult<($($T),*, $TT)>, DeserError> {
        match self.index {
          -1 => tuple_stage_start!(self, token),
          $(
            $i => tuple_stage!(self, token, $SubDeserializer, $R, $T, $i, {
              Err(format!("expected array of length {}, got {}", $ii + 1, $i).into())
            }, {
              Err(format!("expected array of length {}, got {}", $ii + 1, $i).into())
            })
          ),*,
          $ii => tuple_stage!(self, token, $SubDeserializer, $RR, $TT, $ii, {
              Err(format!("expected array of length {}, got {}", $ii + 1, $ii).into())
            }, {
              self.index = -1;
              Ok(DeserResult::Complete(unsafe { self.ret.assume_init_read() }))
            }
          ),
          val if val == $ii + 1 => tuple_stage_end!(self, token),
          _ => unreachable!(),
        }
      }
    }
    impl<$($T),*, $TT> DefaultDeserializable<($($T),*, $TT)> for ($($T),*, $TT)
    where $($T: DefaultDeserializable<$T>),*, $TT: DefaultDeserializable<$TT>
    {
      type DefaultDeserializer = $Deserializer<$($T),*, $TT>;
      fn default_deserializer() -> Self::DefaultDeserializer {
        $Deserializer {
          stage: 0,
          index: -1,
          subdeser: $SubDeserializer::None,
          ret: std::mem::MaybeUninit::uninit(),
        }
      }
    }
    impl<$($T),*, $TT> Drop for $Deserializer<$($T),*, $TT>
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

define_tuple_deserializer! { Tuple1SubDeserializer, Tuple1Deserializer, T1, R1 }
impl<T1> Deserializer<(T1,)> for Tuple1Deserializer<T1>
where
  T1: DefaultDeserializable<T1>,
{
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<(T1,)>, DeserError> {
    match self.index {
      -1 => tuple_stage_start!(self, token),
      0 => tuple_stage!(
        self,
        token,
        Tuple1SubDeserializer,
        R1,
        T1,
        0,
        { Err(format!("expected array of length {}, got {}", 1, 0).into()) },
        {
          self.index = -1;
          Ok(DeserResult::Complete(unsafe { self.ret.assume_init_read() }))
        }
      ),
      1 => tuple_stage_end!(self, token),
      _ => unreachable!(),
    }
  }
}
impl<T1> DefaultDeserializable<(T1,)> for (T1,)
where
  T1: DefaultDeserializable<T1>,
{
  type DefaultDeserializer = Tuple1Deserializer<T1>;
  fn default_deserializer() -> Self::DefaultDeserializer {
    Tuple1Deserializer {
      stage: 0,
      index: -1,
      subdeser: Tuple1SubDeserializer::None,
      ret: std::mem::MaybeUninit::uninit(),
    }
  }
}
impl<T1> Drop for Tuple1Deserializer<T1>
where
  T1: DefaultDeserializable<T1>,
{
  fn drop(&mut self) {
    if self.index > 0 || (self.index == 0 && self.stage == 1) {
      std::mem::drop(unsafe { std::ptr::addr_of_mut!((*self.ret.as_mut_ptr()).0).read() });
    }
  }
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
