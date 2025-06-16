use crate::{
  deserialize::{DeserError, DeserResult, Deserializer, token_is_space},
  stream_parser::{Token, TokenInfo},
};
use std::marker::PhantomData;

pub trait ArrayReceiverTrait<Element, Return, SubDeserializer: Deserializer<Element>> {
  fn create_element(&mut self) -> Result<SubDeserializer, DeserError>;
  fn append(&mut self, element: Element) -> Result<(), DeserError>;
  fn end(&mut self) -> Result<Return, DeserError>;
}

#[derive(Debug, Clone, Copy)]
enum StageEnum {
  NotStarted,
  WaitElement,
  Element,
  ElementEnd,
  End,
}
pub struct ArrayReceiverDeserializer<Element, Return, Receiver, SubDeserializer>
where
  Receiver: ArrayReceiverTrait<Element, Return, SubDeserializer>,
  SubDeserializer: Deserializer<Element>,
{
  receiver: Receiver,
  subreceiver: Option<SubDeserializer>,
  stage: StageEnum,
  _phantom: PhantomData<(Return, Element)>,
}
impl<Element, Return, Receiver, SubDeserializer> Deserializer<Return>
  for ArrayReceiverDeserializer<Element, Return, Receiver, SubDeserializer>
where
  Receiver: ArrayReceiverTrait<Element, Return, SubDeserializer>,
  SubDeserializer: Deserializer<Element>,
{
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<Return>, DeserError> {
    if matches!(self.stage, StageEnum::Element) {
      match self.subreceiver.as_mut().unwrap().feed_token(token)? {
        DeserResult::Complete(elem) => {
          self.subreceiver.take();
          self.receiver.append(elem)?;
          self.stage = StageEnum::ElementEnd;
          return Ok(DeserResult::Continue);
        }
        DeserResult::CompleteWithRollback(elem) => {
          self.subreceiver.take();
          self.receiver.append(elem)?;
          self.stage = StageEnum::ElementEnd;
          // fallthrough
        }
        DeserResult::Continue => {
          return Ok(DeserResult::Continue);
        }
      };
    }

    match token.info {
      TokenInfo::ArrayStart => {
        assert!(matches!(self.stage, StageEnum::NotStarted));
        self.stage = StageEnum::WaitElement;
        Ok(DeserResult::Continue)
      }
      TokenInfo::ArrayEnd => {
        assert!(matches!(self.stage, StageEnum::ElementEnd | StageEnum::WaitElement));
        self.stage = StageEnum::End;
        Ok(DeserResult::Complete(self.receiver.end()?))
      }
      TokenInfo::ArrayNext => {
        assert!(matches!(self.stage, StageEnum::ElementEnd));
        self.stage = StageEnum::WaitElement;
        Ok(DeserResult::Continue)
      }
      _ => {
        if !token_is_space(&token) {
          if !matches!(self.stage, StageEnum::WaitElement) {
            return Err("expect array".into());
          }
          self.subreceiver = Some(self.receiver.create_element()?);
          self.stage = StageEnum::Element;

          if let DeserResult::Complete(elem) =
            self.subreceiver.as_mut().unwrap().feed_token(token)?
          {
            self.subreceiver.take();
            self.receiver.append(elem)?;
            self.stage = StageEnum::ElementEnd;
          }
        }
        Ok(DeserResult::Continue)
      }
    }
  }
}

pub fn create_array_deserializer<Element, Return, Receiver, SubDeserializer>(
  receiver: Receiver,
) -> ArrayReceiverDeserializer<Element, Return, Receiver, SubDeserializer>
where
  Receiver: ArrayReceiverTrait<Element, Return, SubDeserializer>,
  SubDeserializer: Deserializer<Element>,
{
  ArrayReceiverDeserializer {
    receiver,
    subreceiver: None,
    stage: StageEnum::NotStarted,
    _phantom: PhantomData,
  }
}
