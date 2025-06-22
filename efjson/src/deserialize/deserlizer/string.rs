use crate::deserialize::{
  create_string_deserializer, DefaultDeserializable, DeserError, StringReceiverDeserializer,
  StringReceiverTrait,
};

#[derive(Debug)]
pub struct StringReceiver {
  pub string: String,
}

impl StringReceiverTrait<String> for StringReceiver {
  fn start(&mut self) -> Result<(), DeserError> {
    Ok(())
  }
  fn push(&mut self, c: char) -> Result<(), DeserError> {
    self.string.push(c);
    Ok(())
  }
  fn end(&mut self) -> Result<String, DeserError> {
    Ok(std::mem::take(&mut self.string))
  }
}
impl DefaultDeserializable<String> for String {
  type DefaultDeserializer = StringReceiverDeserializer<String, StringReceiver>;
  fn default_deserializer() -> Self::DefaultDeserializer {
    create_string_deserializer(StringReceiver { string: String::new() })
  }
}

impl StringReceiverTrait<Box<str>> for StringReceiver {
  fn start(&mut self) -> Result<(), DeserError> {
    Ok(())
  }
  fn push(&mut self, c: char) -> Result<(), DeserError> {
    self.string.push(c);
    Ok(())
  }
  fn end(&mut self) -> Result<Box<str>, DeserError> {
    Ok(std::mem::take(&mut self.string).into())
  }
}
impl DefaultDeserializable<Box<str>> for Box<str> {
  type DefaultDeserializer = StringReceiverDeserializer<Box<str>, StringReceiver>;
  fn default_deserializer() -> Self::DefaultDeserializer {
    create_string_deserializer(StringReceiver { string: String::new() })
  }
}
