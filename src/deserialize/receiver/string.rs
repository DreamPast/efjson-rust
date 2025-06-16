use crate::{
  deserialize::{DeserError, DeserResult, Deserializer},
  stream_parser::{Token, TokenInfo},
};

pub trait StringReceiverTrait<Return> {
  fn start(&mut self) -> Result<(), DeserError>;
  fn push(&mut self, c: char) -> Result<(), DeserError>;
  fn end(&mut self) -> Result<Return, DeserError>;
}

pub struct StringReceiverDeserializer<Return, Receiver>
where
  Receiver: StringReceiverTrait<Return>,
{
  receiver: Receiver,
  _phantom: std::marker::PhantomData<Return>,
}
impl<Return, Receiver> Deserializer<Return> for StringReceiverDeserializer<Return, Receiver>
where
  Receiver: StringReceiverTrait<Return>,
{
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<Return>, DeserError> {
    match token.info {
      TokenInfo::StringStart => {
        self.receiver.start()?;
        Ok(DeserResult::Continue)
      }
      TokenInfo::StringEnd => Ok(DeserResult::Complete(self.receiver.end()?)),
      TokenInfo::StringNormal => {
        self.receiver.push(token.c)?;
        Ok(DeserResult::Continue)
      }
      TokenInfo::StringEscapeStart
      | TokenInfo::StringEscapeHexStart
      | TokenInfo::StringEscapeUnicodeStart
      | TokenInfo::StringNextLine => Ok(DeserResult::Continue),
      TokenInfo::StringEscape(c) => {
        self.receiver.push(c)?;
        Ok(DeserResult::Continue)
      }
      TokenInfo::StringEscapeUnicode(_, c) | TokenInfo::StringEscapeHex(_, c) => {
        if let Some(c) = c {
          self.receiver.push(c)?;
        }
        Ok(DeserResult::Continue)
      }
      _ => {
        if token.is_space() {
          Ok(DeserResult::Continue)
        } else {
          Err("expect string".into())
        }
      }
    }
  }
}

pub fn create_string_deserializer<Return, Receiver>(
  receiver: Receiver,
) -> StringReceiverDeserializer<Return, Receiver>
where
  Receiver: StringReceiverTrait<Return>,
{
  StringReceiverDeserializer { receiver, _phantom: std::marker::PhantomData }
}
