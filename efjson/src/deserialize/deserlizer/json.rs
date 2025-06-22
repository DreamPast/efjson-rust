use crate::{
  deserialize::{DefaultDeserializable, DeserError, DeserResult, Deserializer},
  stream_parser::{Category, Token},
  JsonValue,
};

#[derive(Debug)]
enum JsonSubdeserializer {
  Null(<() as DefaultDeserializable<()>>::DefaultDeserializer),
  Boolean(<bool as DefaultDeserializable<bool>>::DefaultDeserializer),
  Number(<f64 as DefaultDeserializable<f64>>::DefaultDeserializer),
  String(<String as DefaultDeserializable<String>>::DefaultDeserializer),
  Array(<Vec<JsonValue> as DefaultDeserializable<Vec<JsonValue>>>::DefaultDeserializer),
  Object(
    <std::collections::HashMap<String, JsonValue> as DefaultDeserializable<
      std::collections::HashMap<String, JsonValue>,
    >>::DefaultDeserializer,
  ),
}
#[derive(Debug)]
pub struct JsonDeserializer {
  subdeser: Option<Box<JsonSubdeserializer>>,
}
impl Deserializer<JsonValue> for JsonDeserializer {
  fn feed_token(&mut self, token: Token) -> Result<DeserResult<JsonValue>, DeserError> {
    if self.subdeser.is_none() {
      match token.info.get_category() {
        Category::Whitespace | Category::Eof | Category::Comment => {
          return Ok(DeserResult::Continue);
        }
        Category::Null => {
          self.subdeser = Some(Box::new(JsonSubdeserializer::Null(<() as DefaultDeserializable<
            (),
          >>::default_deserializer(
          ))))
        }
        Category::Boolean => {
          self.subdeser = Some(Box::new(JsonSubdeserializer::Boolean(
            <bool as DefaultDeserializable<bool>>::default_deserializer(),
          )))
        }
        Category::Number => {
          self.subdeser = Some(Box::new(JsonSubdeserializer::Number(
            <f64 as DefaultDeserializable<f64>>::default_deserializer(),
          )))
        }
        Category::String => {
          self.subdeser = Some(Box::new(JsonSubdeserializer::String(
            <String as DefaultDeserializable<String>>::default_deserializer(),
          )))
        }
        Category::Object => {
          self.subdeser = Some(Box::new(JsonSubdeserializer::Object(<std::collections::HashMap<
            String,
            JsonValue,
          > as DefaultDeserializable<
            std::collections::HashMap<String, JsonValue>,
          >>::default_deserializer(
          ))))
        }
        Category::Array => {
          self.subdeser =
            Some(Box::new(JsonSubdeserializer::Array(<Vec<JsonValue> as DefaultDeserializable<
              Vec<JsonValue>,
            >>::default_deserializer())))
        }
        Category::Identifier => {
          return Err("unexpected identifier".into());
        }
      }
    }
    Ok(match self.subdeser.as_mut().unwrap().as_mut() {
      JsonSubdeserializer::Null(deser) => deser.feed_token(token)?.map(|_| JsonValue::Null),
      JsonSubdeserializer::Boolean(deser) => {
        deser.feed_token(token)?.map(|v| JsonValue::Boolean(v))
      }
      JsonSubdeserializer::Number(deser) => deser.feed_token(token)?.map(|v| JsonValue::Number(v)),
      JsonSubdeserializer::String(deser) => deser.feed_token(token)?.map(|v| JsonValue::String(v)),
      JsonSubdeserializer::Array(deser) => deser.feed_token(token)?.map(|v| JsonValue::Array(v)),
      JsonSubdeserializer::Object(deser) => deser.feed_token(token)?.map(|v| JsonValue::Object(v)),
    })
  }
}
impl DefaultDeserializable<JsonValue> for JsonValue {
  type DefaultDeserializer = JsonDeserializer;
  fn default_deserializer() -> Self::DefaultDeserializer {
    JsonDeserializer { subdeser: None }
  }
}
