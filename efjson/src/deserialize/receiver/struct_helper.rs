use std::{marker::PhantomData, mem::MaybeUninit};

use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer},
  stream_parser::{Token, TokenInfo},
};

pub trait StructHelperReceiverTrait<Return> {
  fn start_value(&mut self, key: &str) -> Result<(), DeserError>;
  fn feed_value(&mut self, token: Token) -> Result<DeserResult<()>, DeserError>;
  fn end(&mut self) -> Result<Return, DeserError>;
}

#[derive(Debug, Clone, Copy)]
enum StageEnum {
  NotStarted,
  WaitKey,
  Key,
  KeyEnd,
  WaitValue,
  Value,
  ValueEnd,
  End,
}
type StringDeserializer = <String as DefaultDeserializable<String>>::DefaultDeserializer;

#[derive(Debug)]
pub struct StructHelperReceiverDeserializer<Return, Receiver: StructHelperReceiverTrait<Return>> {
  receiver: Receiver,
  key_subreceiver: MaybeUninit<StringDeserializer>,
  key: MaybeUninit<String>,
  stage: StageEnum,
  _phantom: PhantomData<Return>,
}
impl<Return, Receiver: StructHelperReceiverTrait<Return>> Deserializer<Return>
  for StructHelperReceiverDeserializer<Return, Receiver>
{
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<Return>, DeserError> {
    match self.stage {
      StageEnum::Key => {
        match unsafe { self.key_subreceiver.assume_init_mut() }.feed_token(token)? {
          DeserResult::Complete(key) => {
            unsafe { self.key_subreceiver.assume_init_drop() };
            self.key.write(key);
            self.stage = StageEnum::KeyEnd;
            return Ok(DeserResult::Continue);
          }
          DeserResult::CompleteWithRollback(key) => {
            unsafe { self.key_subreceiver.assume_init_drop() };
            self.key.write(key);
            self.stage = StageEnum::KeyEnd;
            // fallthrough
          }
          DeserResult::Continue => return Ok(DeserResult::Continue),
        }
      }
      StageEnum::Value => {
        match self.receiver.feed_value(token)? {
          DeserResult::Complete(_) => {
            self.stage = StageEnum::ValueEnd;
            return Ok(DeserResult::Continue);
          }
          DeserResult::CompleteWithRollback(_) => {
            self.stage = StageEnum::ValueEnd;
            // fallthrough
          }
          DeserResult::Continue => return Ok(DeserResult::Continue),
        }
      }
      _ => {}
    }
    if matches!(self.stage, StageEnum::WaitKey) {
      if !token.is_space() {
        if matches!(token.info, TokenInfo::ObjectEnd) {
          // trailing comma
          self.stage = StageEnum::End;
          return Ok(DeserResult::Complete(self.receiver.end()?));
        }

        self
          .key_subreceiver
          .write(<String as DefaultDeserializable<String>>::default_deserializer());
        self.stage = StageEnum::Key;

        match unsafe { self.key_subreceiver.assume_init_mut() }.feed_token(token)? {
          DeserResult::Complete(key) => {
            unsafe { self.key_subreceiver.assume_init_drop() };
            self.key.write(key);
            self.stage = StageEnum::KeyEnd;
          }
          DeserResult::CompleteWithRollback(_) => unreachable!(),
          DeserResult::Continue => {}
        }
      }
      return Ok(DeserResult::Continue);
    }
    if matches!(self.stage, StageEnum::WaitValue) {
      if !token.is_space() {
        self.stage = StageEnum::Value;
        self.receiver.start_value(unsafe { self.key.assume_init_ref() })?;
        match self.receiver.feed_value(token)? {
          DeserResult::Complete(_) => self.stage = StageEnum::ValueEnd,
          DeserResult::CompleteWithRollback(_) => unreachable!(),
          DeserResult::Continue => {}
        }
      }
      return Ok(DeserResult::Continue);
    }

    match token.info {
      TokenInfo::ObjectStart => {
        assert!(matches!(self.stage, StageEnum::NotStarted));
        self.stage = StageEnum::WaitKey;
        Ok(DeserResult::Continue)
      }
      TokenInfo::ObjectEnd => {
        assert!(matches!(self.stage, StageEnum::ValueEnd));
        self.stage = StageEnum::End;
        Ok(DeserResult::Complete(self.receiver.end()?))
      }
      TokenInfo::ObjectValueStart => {
        self.stage = StageEnum::WaitValue;
        Ok(DeserResult::Continue)
      }
      TokenInfo::ObjectNext => {
        assert!(matches!(self.stage, StageEnum::ValueEnd));
        self.stage = StageEnum::WaitKey;
        Ok(DeserResult::Continue)
      }
      _ => {
        if token.is_space() {
          Ok(DeserResult::Continue)
        } else {
          Err("expect object".into())
        }
      }
    }
  }
}

impl<Return, Receiver: StructHelperReceiverTrait<Return>> Drop
  for StructHelperReceiverDeserializer<Return, Receiver>
{
  fn drop(&mut self) {
    if matches!(self.stage, StageEnum::Key) {
      unsafe { self.key_subreceiver.assume_init_drop() };
    }
    if matches!(self.stage, StageEnum::KeyEnd | StageEnum::WaitValue | StageEnum::Value) {
      unsafe { self.key.assume_init_drop() };
    }
  }
}

pub fn create_struct_helper_deserializer<Return, Receiver: StructHelperReceiverTrait<Return>>(
  receiver: Receiver,
) -> StructHelperReceiverDeserializer<Return, Receiver> {
  StructHelperReceiverDeserializer {
    receiver,
    key: MaybeUninit::uninit(),
    key_subreceiver: MaybeUninit::uninit(),
    stage: StageEnum::NotStarted,
    _phantom: PhantomData,
  }
}
