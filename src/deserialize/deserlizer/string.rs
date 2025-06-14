use crate::deserialize::{
  DefaultDeserializable, DeserError, Deserializer, StringReceiverTrait, create_string_deserializer,
};

#[derive(Debug)]
struct StringReceiver {
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
  fn default_deserializer() -> impl Deserializer<String> {
    create_string_deserializer(StringReceiver { string: String::new() })
  }
}
