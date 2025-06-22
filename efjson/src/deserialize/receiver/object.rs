use crate::{
  deserialize::{DeserError, DeserResult, Deserializer},
  stream_parser::{Token, TokenInfo},
};
use std::{marker::PhantomData, mem::MaybeUninit};

pub trait ObjectReceiverTrait<
  Key,
  Value,
  Return,
  KeyDeserializer: Deserializer<Key>,
  ValueDeserializer: Deserializer<Value>,
>
{
  fn create_key(&mut self) -> Result<KeyDeserializer, DeserError>;
  fn create_value(&mut self, key: &Key) -> Result<ValueDeserializer, DeserError>;
  fn set(&mut self, key: Key, value: Value) -> Result<(), DeserError>;
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
#[derive(Debug)]
pub struct ObjectReceiverDeserializer<
  Key,
  Value,
  Return,
  Receiver,
  KeyDeserializer,
  ValueDeserializer,
> where
  Receiver: ObjectReceiverTrait<Key, Value, Return, KeyDeserializer, ValueDeserializer>,
  KeyDeserializer: Deserializer<Key>,
  ValueDeserializer: Deserializer<Value>,
{
  receiver: Receiver,
  key_subreceiver: MaybeUninit<KeyDeserializer>,
  value_subreceiver: MaybeUninit<ValueDeserializer>,
  key: MaybeUninit<Key>,
  stage: StageEnum,
  _phantom: PhantomData<(Return, Value)>,
}
impl<Key, Value, Return, Receiver, KeyDeserializer, ValueDeserializer> Deserializer<Return>
  for ObjectReceiverDeserializer<Key, Value, Return, Receiver, KeyDeserializer, ValueDeserializer>
where
  Receiver: ObjectReceiverTrait<Key, Value, Return, KeyDeserializer, ValueDeserializer>,
  KeyDeserializer: Deserializer<Key>,
  ValueDeserializer: Deserializer<Value>,
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
        match unsafe { self.value_subreceiver.assume_init_mut() }.feed_token(token)? {
          DeserResult::Complete(value) => {
            unsafe { self.value_subreceiver.assume_init_drop() };
            self.stage = StageEnum::ValueEnd;
            self.receiver.set(unsafe { self.key.assume_init_read() }, value)?;
            return Ok(DeserResult::Continue);
          }
          DeserResult::CompleteWithRollback(value) => {
            unsafe { self.value_subreceiver.assume_init_drop() };
            self.stage = StageEnum::ValueEnd;
            self.receiver.set(unsafe { self.key.assume_init_read() }, value)?;
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

        self.key_subreceiver.write(self.receiver.create_key()?);
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
        self
          .value_subreceiver
          .write(self.receiver.create_value(unsafe { self.key.assume_init_ref() })?);

        match unsafe { self.value_subreceiver.assume_init_mut() }.feed_token(token)? {
          DeserResult::Complete(value) => {
            unsafe { self.value_subreceiver.assume_init_drop() };
            self.stage = StageEnum::ValueEnd;
            self.receiver.set(unsafe { self.key.assume_init_read() }, value)?;
          }
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

impl<Key, Value, Return, Receiver, KeyDeserializer, ValueDeserializer> Drop
  for ObjectReceiverDeserializer<Key, Value, Return, Receiver, KeyDeserializer, ValueDeserializer>
where
  Receiver: ObjectReceiverTrait<Key, Value, Return, KeyDeserializer, ValueDeserializer>,
  KeyDeserializer: Deserializer<Key>,
  ValueDeserializer: Deserializer<Value>,
{
  fn drop(&mut self) {
    if matches!(self.stage, StageEnum::Key) {
      unsafe { self.key_subreceiver.assume_init_drop() };
    }
    if matches!(self.stage, StageEnum::Value) {
      unsafe { self.value_subreceiver.assume_init_drop() };
    }
    if matches!(self.stage, StageEnum::KeyEnd | StageEnum::WaitValue | StageEnum::Value) {
      unsafe { self.key.assume_init_drop() };
    }
  }
}

pub fn create_object_deserializer<
  Key,
  Value,
  Return,
  Receiver,
  KeyDeserializer,
  ValueDeserializer,
>(
  receiver: Receiver,
) -> ObjectReceiverDeserializer<Key, Value, Return, Receiver, KeyDeserializer, ValueDeserializer>
where
  Receiver: ObjectReceiverTrait<Key, Value, Return, KeyDeserializer, ValueDeserializer>,
  KeyDeserializer: Deserializer<Key>,
  ValueDeserializer: Deserializer<Value>,
{
  ObjectReceiverDeserializer {
    receiver,
    key: MaybeUninit::uninit(),
    key_subreceiver: MaybeUninit::uninit(),
    value_subreceiver: MaybeUninit::uninit(),
    stage: StageEnum::NotStarted,
    _phantom: PhantomData,
  }
}
