use crate::{
  deserialize::{DeserError, DeserResult, Deserializer, token_is_space},
  stream_parser::{Token, TokenInfo},
};
use std::marker::PhantomData;

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
  key_subreceiver: Option<KeyDeserializer>,
  value_subreceiver: Option<ValueDeserializer>,
  key: Option<Key>,
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
      StageEnum::Key => match self.key_subreceiver.as_mut().unwrap().feed_token(token)? {
        DeserResult::Complete(key) => {
          self.key_subreceiver.take();
          self.key = Some(key);
          self.stage = StageEnum::KeyEnd;
          return Ok(DeserResult::Continue);
        }
        DeserResult::CompleteWithRollback(key) => {
          self.key_subreceiver.take();
          self.key = Some(key);
          self.stage = StageEnum::KeyEnd;
          // fallthrough
        }
        DeserResult::Continue => {
          return Ok(DeserResult::Continue);
        }
      },
      StageEnum::Value => match self.value_subreceiver.as_mut().unwrap().feed_token(token)? {
        DeserResult::Complete(value) => {
          self.value_subreceiver.take();
          self.stage = StageEnum::ValueEnd;
          self.receiver.set(self.key.take().unwrap(), value)?;
          return Ok(DeserResult::Continue);
        }
        DeserResult::CompleteWithRollback(value) => {
          self.value_subreceiver.take();
          self.stage = StageEnum::ValueEnd;
          self.receiver.set(self.key.take().unwrap(), value)?;
          // fallthrough
        }
        DeserResult::Continue => {
          return Ok(DeserResult::Continue);
        }
      },
      _ => {}
    }
    match token.info {
      TokenInfo::ObjectStart => {
        assert!(matches!(self.stage, StageEnum::NotStarted));
        self.stage = StageEnum::WaitKey;
        Ok(DeserResult::Continue)
      }
      TokenInfo::ObjectEnd => {
        assert!(matches!(self.stage, StageEnum::ValueEnd | StageEnum::WaitKey));
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
        if !token_is_space(&token) {
          match self.stage {
            StageEnum::WaitKey => {
              self.stage = StageEnum::Key;
              self.key_subreceiver = Some(self.receiver.create_key()?);

              if let DeserResult::Complete(key) =
                self.key_subreceiver.as_mut().unwrap().feed_token(token)?
              {
                self.key_subreceiver.take();
                self.key = Some(key);
                self.stage = StageEnum::KeyEnd;
              }
            }
            StageEnum::WaitValue => {
              self.stage = StageEnum::Value;
              self.value_subreceiver =
                Some(self.receiver.create_value(self.key.as_ref().unwrap())?);

              if let DeserResult::Complete(value) =
                self.value_subreceiver.as_mut().unwrap().feed_token(token)?
              {
                self.value_subreceiver.take();
                self.stage = StageEnum::ValueEnd;
                self.receiver.set(self.key.take().unwrap(), value)?;
              }
            }
            _ => {
              return Err("expect object".into());
            }
          }
        }
        Ok(DeserResult::Continue)
      }
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
    key: None,
    key_subreceiver: None,
    value_subreceiver: None,
    stage: StageEnum::NotStarted,
    _phantom: PhantomData,
  }
}
